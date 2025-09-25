# Command Palette & Actor API Architecture

## Overview

This document describes the scalable architecture for the command palette system and actor API interface. The system is designed to handle thousands of commands across hundreds of actors while maintaining performance and extensibility.

## Architecture Components

### 1. Actor API System

#### Core Traits

```rust
pub trait ActorAPI {
    fn actor_type(&self) -> String;
    fn get_api_methods(&self) -> Vec<ApiMethod>;
    fn execute_api_method(&mut self, method: &str, params: ApiParams) -> Result<ApiResult>;
    fn can_handle_method(&self, method: &str) -> bool;
    fn get_capabilities(&self) -> Vec<String>;
    fn get_state(&self) -> HashMap<String, serde_json::Value>;
}

pub trait Actor: Send + Sync + ActorAPI {
    // Standard actor methods...
}
```

#### Scalability Features

**Dynamic Method Discovery**: Actor API methods are discovered at runtime, allowing actors to expose new functionality without changing the command palette core.

**Type-Safe Parameters**: All API calls use strongly-typed parameters through `ApiParams` with JSON serialization/deserialization.

**Capability-Based Grouping**: Actors expose capabilities that allow the command palette to intelligently group and filter commands.

### 2. Command Palette System

#### Headless Core Design

The command palette is designed as a headless service with optional UI components:

```rust
pub struct CommandPalette {
    providers: HashMap<String, Box<dyn CommandProvider>>,
    groups: HashMap<String, CommandGroup>,
    custom_activators: HashMap<String, Box<dyn Fn(&CommandContext) -> bool + Send + Sync>>,
    // UI state is separate
}
```

#### Command Providers

**Global Provider**: IDE-wide commands (file operations, view management)
**Actor Provider**: Dynamically generated from actor APIs
**Custom Providers**: Plugin or extension-provided commands

#### Performance Optimizations

1. **Lazy Loading**: Commands are generated on-demand when the palette is opened
2. **Caching**: API method lists are cached per actor type
3. **Incremental Search**: Search results are computed incrementally as user types
4. **Provider Filtering**: Only active providers are queried based on context

### 3. Command Execution Flow

```
User Input -> Command Search -> Context Filtering -> Execution Routing -> Result Handling
```

1. **User Input**: Search query or direct command invocation
2. **Command Search**: Fuzzy matching across all active providers
3. **Context Filtering**: Commands filtered by current focus, view system, etc.
4. **Execution Routing**: Commands routed to appropriate handler (global, actor, custom)
5. **Result Handling**: Success/error feedback and UI updates

## Scalability Considerations

### Memory Management

**Command Caching Strategy**:
- Commands are cached per context combination (focused actor + view system)
- Cache is invalidated when actors are added/removed or context changes
- Maximum cache size limits prevent unbounded growth

**Provider Registration**:
- Providers are registered once and reused
- Dynamic provider addition/removal supported for plugins

### Performance Characteristics

**Command Discovery**: O(P) where P = number of active providers
**Search Performance**: O(C × log C) where C = number of cached commands
**Memory Usage**: O(A × M) where A = actors, M = average methods per actor

### Concurrency

**Thread Safety**:
- Command palette core is `Send + Sync`
- Actor API calls are synchronous but can be made async if needed
- UI rendering happens on main thread only

**Actor Isolation**:
- Each actor maintains its own API state
- No shared mutable state between actors
- Command execution is isolated per actor

## Configuration & Extensibility

### Command Groups

Commands are organized into logical groups with activation contexts:

```rust
pub enum GroupActivation {
    Always,
    ActorType(String),
    ActorGroup(Vec<String>),
    ViewSystem(String),
    Custom(String), // Custom activator function
}
```

Groups can be configured to activate based on:
- Current actor type or group of actor types
- Active view system (scene, tiling, etc.)
- Custom logic (lambda functions)

### Actor Integration Pattern

To integrate a new actor type:

1. **Implement ActorAPI**:
```rust
impl ActorAPI for MyActor {
    fn actor_type(&self) -> String { "MyActor".to_string() }

    fn get_api_methods(&self) -> Vec<ApiMethod> {
        vec![/* define your methods */]
    }

    fn execute_api_method(&mut self, method: &str, params: ApiParams) -> Result<ApiResult> {
        match method {
            "my_method" => { /* implementation */ },
            _ => Err(anyhow!("Unknown method"))
        }
    }
}
```

2. **Register with ActorManager**: The command palette automatically discovers the actor's API.

### Plugin System Integration

The architecture supports plugins through:

**Custom Command Providers**:
```rust
struct MyPluginProvider { /* plugin state */ }
impl CommandProvider for MyPluginProvider { /* implementation */ }

// Register with palette
palette.register_provider(Box::new(MyPluginProvider::new()));
```

**Custom Group Activators**:
```rust
palette.add_custom_activator("my_condition".to_string(), Box::new(|ctx| {
    // Custom activation logic
}));
```

## Usage Examples

### Basic Command Execution

```rust
// Execute command by ID
let result = palette.execute_command("file.new", &context)?;

// Search and execute
let commands = palette.search_commands("format", &context);
if let Some(cmd) = commands.first() {
    palette.execute_command(&cmd.id, &context)?;
}
```

### Actor API Calls

```rust
// Direct API call
let params = ApiParams::new()
    .with_param("content", "Hello, world!");
actor_manager.execute_actor_api(actor_id, "set_content", params)?;

// Get actor information
let info = actor_manager.get_actors_info();
for actor in info {
    println!("Actor: {} ({})", actor.name, actor.actor_type);
    println!("Capabilities: {:?}", actor.capabilities);
}
```

### Command Groups

```rust
// Find actors with specific capability
let text_editors = actor_manager.find_actors_with_capability("text_editing");

// Commands will be grouped automatically by actor type and capabilities
let commands = palette.get_available_commands(&context);
```

## Error Handling

### API Errors

- **Parameter Validation**: Type checking and required parameter validation
- **Method Resolution**: Clear errors for unknown methods or actors
- **Execution Errors**: Actor-specific error propagation

### Command Palette Errors

- **Search Failures**: Graceful degradation with partial results
- **Provider Errors**: Isolated failure - other providers continue working
- **Context Errors**: Default to global commands if context invalid

## Testing Strategy

### Unit Tests
- Actor API method discovery and execution
- Command provider registration and querying
- Parameter serialization/deserialization
- Group activation logic

### Integration Tests
- Full command palette workflow
- Actor-to-command-palette integration
- Multi-actor command coordination
- Performance with large numbers of actors/commands

### Performance Tests
- Command search with 10,000+ commands
- Actor API calls with complex parameters
- Memory usage under load
- Concurrent access patterns

## Future Enhancements

### Planned Features

1. **Async API Methods**: Support for long-running actor operations
2. **Command Macros**: Ability to chain multiple commands
3. **Fuzzy Search Improvements**: Better ranking algorithms
4. **Command History**: Recently used commands tracking
5. **Keyboard Navigation**: Full keyboard control of command palette

### Extension Points

1. **Custom Search Algorithms**: Pluggable search implementations
2. **Command Validation**: Pre-execution validation framework
3. **Result Processors**: Custom handling of command results
4. **Context Providers**: Additional context sources for command filtering

## API Reference

### Core Types

- `Command`: Individual executable action with metadata
- `CommandProvider`: Source of commands with context awareness
- `CommandGroup`: Logical grouping with activation rules
- `ApiMethod`: Actor method definition with parameters
- `ApiParams`: Type-safe parameter container
- `ApiResult`: Execution result with error handling

### Key Methods

- `CommandPalette::search_commands()`: Find commands by query
- `CommandPalette::execute_command()`: Execute command by ID
- `ActorManager::execute_actor_api()`: Direct actor API calls
- `ActorManager::find_actors_by_type()`: Query actors by type

This architecture scales to handle:
- **1000+ actors** with isolated API namespaces
- **10,000+ commands** with efficient search and caching
- **100+ command providers** with lazy loading
- **Custom activation logic** without core modifications
- **Plugin ecosystem** through provider registration
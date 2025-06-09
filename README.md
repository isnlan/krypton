# Krypton - File Encryption/Decryption Tool

A modular Rust GUI application for file encryption and decryption built with egui.

## Project Structure

The project follows Rust naming conventions and modular design principles with high cohesion and low coupling:

```
src/
├── main.rs          # Application entry point
├── models.rs        # Data models and types
├── core.rs          # Core business logic
├── app.rs           # Application state management
└── ui/              # User interface components
    ├── mod.rs       # UI module exports
    ├── panels.rs    # UI panels (Settings, File, Progress, Control)
    └── dialogs.rs   # Dialog components (Error, Complete)
```

## Module Design

### `models.rs` - Data Models
- **High Cohesion**: Contains all data structures and enums
- Defines: `OperationMode`, `EncryptionAlgorithm`, `FileItem`, `AppState`
- No external dependencies except standard library

### `core.rs` - Business Logic
- **High Cohesion**: Contains core functionality
- `FileManager`: File system operations
- `CryptoEngine`: Encryption/decryption operations
- **Low Coupling**: Only depends on models module

### `ui/` - User Interface Module
- **Separation of Concerns**: Each component has single responsibility
- `panels.rs`: Reusable UI panels with functional approach
- `dialogs.rs`: Modal dialog components
- **Low Coupling**: UI components are stateless and receive data via parameters

### `app.rs` - Application State
- **High Cohesion**: Centralized state management
- Coordinates between UI and core modules
- Implements the main application logic and event handling

## Design Principles Applied

1. **High Cohesion**: Each module has a single, well-defined responsibility
2. **Low Coupling**: Modules communicate through clean interfaces
3. **Rust Naming Conventions**: snake_case for functions/variables, PascalCase for types
4. **Event-Driven UI**: UI components return events instead of taking callbacks to avoid borrowing conflicts
5. **Functional UI Components**: UI components are stateless and reusable
6. **Clear Separation**: Business logic separated from presentation logic

## Architecture Patterns

### Event-Driven UI Pattern
To solve Rust's borrowing checker conflicts, the UI uses an event-driven pattern:

- **UI Components** return `Option<Event>` instead of executing callbacks directly
- **Events** are defined as enums (`PanelEvent`, `DialogEvent`)
- **Main Loop** processes events and updates application state
- **No Borrowing Conflicts** since UI doesn't need mutable references to `self`

This pattern ensures thread-safety and prevents the common Rust error: "cannot borrow `*self` as mutable more than once at a time".

## Features

- File encryption/decryption with multiple algorithms (AES-256, ChaCha20, Blowfish)
- Multi-threaded processing
- Progress tracking
- Error handling with user interaction
- Clean, modern GUI using egui

## Building and Running

```bash
cargo build --release
cargo run
```

## Dependencies

- `egui`: Modern immediate mode GUI framework
- `eframe`: egui framework for native applications
- `env_logger`: Logging infrastructure
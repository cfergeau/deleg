# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Tech Stack

- **Backend**: Rust with Rocket web framework
- **Database**: SQLite with sqlx for async database access
- **Frontend**: htmx for dynamic interactions (no separate JS framework)

## Development Commands

### Building and Running

```bash
# Build the project
cargo build

# Run the application
cargo run

# Run with release optimizations
cargo run --release

# Watch for changes and auto-rebuild (requires cargo-watch)
cargo watch -x run
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check

# Run clippy lints
cargo clippy

# Run clippy with all features and fail on warnings
cargo clippy -- -D warnings
```

### Database Management with sqlx

```bash
# Create database (if it doesn't exist)
sqlx database create

# Run migrations
sqlx migrate run

# Create a new migration
sqlx migrate add <migration_name>

# Revert last migration
sqlx migrate revert

# Generate query metadata for compile-time verification (required for sqlx macros)
cargo sqlx prepare
```

## Architecture Notes

### Database Layer (sqlx)

- Use `sqlx::query!` and `sqlx::query_as!` macros for compile-time checked queries
- The `DATABASE_URL` environment variable must be set (e.g., `DATABASE_URL=sqlite:./database.db`)
- Migrations are located in `migrations/` directory
- For offline builds (CI), run `cargo sqlx prepare` to generate `.sqlx/` metadata

### Web Layer (Rocket)

- Routes are async functions returning Rocket responders
- Use Rocket's request guards for authentication, database connections, etc.
- State management through Rocket's managed state
- Configuration via `Rocket.toml` (development/production profiles)

### Frontend (htmx)

- Server renders HTML fragments/pages
- htmx attributes handle dynamic updates without full page reloads
- Return HTML partials from endpoints that htmx will swap into the DOM
- Keep JavaScript minimal - prefer htmx attributes for interactivity

### Project Structure

When organizing the codebase:
- Keep route handlers focused on request/response logic
- Extract business logic into separate modules
- Database queries should be in their own functions/modules for reusability
- HTML templates should be separate from route logic (consider using a template engine like Tera or handlebars)

## Environment Setup

Required environment variables:
- `DATABASE_URL`: Path to SQLite database (e.g., `sqlite:./database.db`)
- `ROCKET_SECRET_KEY`: Secret key for Rocket sessions (generate with `openssl rand -base64 32`)

## Workflow Notes

1. **Database changes**: Always create migrations for schema changes, never modify the database directly
2. **After pulling changes**: Run `sqlx migrate run` to apply new migrations
3. **Compile-time query checking**: If sqlx macro queries fail to compile, ensure the database is up-to-date and run `cargo sqlx prepare`

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Application Purpose

This application tracks delegation hours for members of French works councils (CSE - Comité Social et Économique) in companies with 50+ employees. Different roles receive different amounts of monthly delegation hours as defined by French labor law (Code du travail).

### French CSE Context

The CSE (Comité Social et Économique) is a mandatory employee representation body in French companies with 50+ employees. Members receive monthly delegation hours to fulfill their duties. Key roles include:

- **Élu titulaire CSE**: Regular elected member (18-24h/month depending on company size)
- **Élu suppléant CSE**: Alternate member (0h by default, unless replacing a titulaire)
- **Délégué syndical (DS)**: Union delegate (12-24h/month based on company size)
- **Représentant syndical au CSE (RS)**: Union representative at CSE (20h/month for 500+ employees)
- **Représentant de section syndicale (RSS)**: Union section representative (4h/month)
- **Membre CSSCT**: Health & Safety Commission member (hours per company agreement)

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

#### Database Schema

The application uses three main tables:

**persons**
- `id`: INTEGER PRIMARY KEY
- `name`: TEXT (first name)
- `surname`: TEXT (last name)

**roles**
- `id`: INTEGER PRIMARY KEY
- `name`: TEXT UNIQUE (role name, e.g., "Élu titulaire CSE")
- `delegation_hours`: REAL (theoretical monthly delegation hours, default 0.0)

**person_roles** (junction table for many-to-many relationship)
- `person_id`: INTEGER (foreign key to persons, CASCADE DELETE)
- `role_id`: INTEGER (foreign key to roles, CASCADE DELETE)
- PRIMARY KEY: (person_id, role_id)

#### Important Conventions

- **Auto-creating roles**: When assigning roles to a person, if a role doesn't exist, it's automatically created with 0.0 delegation hours. This allows flexibility while ensuring referential integrity.
- **Role names**: Use exact French role names (e.g., "Délégué syndical", "Élu titulaire CSE") to match legal terminology.
- **Delegation hours**: Stored as theoretical/legal values. Actual usage tracking would require additional tables.

### Web Layer (Rocket)

- Routes are async functions returning Rocket responders
- Use Rocket's request guards for authentication, database connections, etc.
- State management through Rocket's managed state
- Configuration via `Rocket.toml` (development/production profiles)

#### REST API Endpoints

**Person Management:**
- `GET /api/persons` - List all persons with their roles
- `GET /api/persons/<id>` - Get a specific person with roles
- `POST /api/persons` - Create a new person with roles
- `PUT /api/persons/<id>` - Update a person and their roles
- `DELETE /api/persons/<id>` - Delete a person

**Role Management:**
- `GET /api/roles` - List all roles
- `GET /api/roles/<id>` - Get a specific role
- `POST /api/roles` - Create a new role
- `PUT /api/roles/<id>` - Update a role
- `DELETE /api/roles/<id>` - Delete a role

**Web Pages:**
- `GET /people` - Display list of all persons
- `GET /people/<id>` - Edit person page with form

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

## Testing

The project has comprehensive test coverage:
- **Database tests** (`src/db.rs`): Test all CRUD operations using in-memory SQLite databases
- **API tests** (`src/routes.rs`): Test all HTTP endpoints with Rocket's test client
- **Template tests**: Verify HTML rendering and htmx integration

When making changes:
- Always run `cargo test` to ensure all tests pass
- Update test data when modifying database schema (especially in `setup_test_db()` and `setup_test_rocket()`)
- When changing the `Role` model, update all `Role::new()` calls in tests to match the new signature

## Workflow Notes

1. **Database changes**: Always create migrations for schema changes, never modify the database directly
2. **After pulling changes**: Run `sqlx migrate run` to apply new migrations
3. **Compile-time query checking**: If sqlx macro queries fail to compile, ensure the database is up-to-date and run `cargo sqlx prepare`
4. **Test-driven development**: Run tests frequently during development to catch regressions early

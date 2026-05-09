# Deleg

A web application for tracking delegation hours for French works council (CSE - Comité Social et Économique) members. Built with Rust, Rocket, and htmx.

## Features

- Track persons with multiple role assignments
- Manage roles with delegation hour allocations
- Date-based role assignments (start and end dates)
- Automatic filtering of expired and future roles
- View all persons and roles in clean table interfaces
- RESTful API for person and role management
- Server-side rendering with Tera templates
- Dynamic interactions with htmx
- SQLite database with sqlx for type-safe queries
- Comprehensive test coverage (44 tests)

## Tech Stack

- **Backend**: Rust with Rocket web framework
- **Database**: SQLite with sqlx for async database access
- **Frontend**: htmx for dynamic interactions
- **Templates**: Tera template engine
- **Testing**: Rocket's test client with in-memory SQLite

## Prerequisites

- Rust (1.70 or later)
- SQLite
- sqlx-cli (for database migrations)

Install sqlx-cli:
```bash
cargo install sqlx-cli --no-default-features --features sqlite
```

## Setup

1. Clone the repository:
```bash
git clone <repository-url>
cd deleg
```

2. Set up the database:
```bash
# Create the database
sqlx database create

# Run migrations
sqlx migrate run
```

3. Build the project:
```bash
cargo build
```

## Running the Application

Start the server:
```bash
cargo run
```

The application will be available at `http://localhost:8000`

### Available Routes

**Web Pages:**
- `GET /people` - View all persons with their currently active roles
- `GET /people/<id>` - Edit a person's information and role assignments
- `GET /roles` - View all roles with delegation hours

**API Endpoints:**

For complete API documentation including request/response formats and examples, see **[API.md](API.md)**.

Summary of endpoints:
- **Persons:** GET, POST, PUT, DELETE `/api/persons` and `/api/persons/<id>`
- **Roles:** GET, POST, PUT, DELETE `/api/roles` and `/api/roles/<id>`

## Development

### Running Tests

Run all tests:
```bash
cargo test
```

Run tests with output:
```bash
cargo test -- --nocapture
```

### Code Quality

Format code:
```bash
cargo fmt
```

Run linter:
```bash
cargo clippy
```

### Database Migrations

Create a new migration:
```bash
sqlx migrate add <migration_name>
```

Run migrations:
```bash
sqlx migrate run
```

Revert last migration:
```bash
sqlx migrate revert
```

## Project Structure

```
deleg/
├── migrations/          # Database migration files
├── src/
│   ├── main.rs         # Application entry point
│   ├── models.rs       # Data models (Person, Role, RoleAssignment)
│   ├── db.rs           # Database operations (CRUD)
│   └── routes.rs       # HTTP routes and handlers
├── templates/          # Tera HTML templates
│   ├── people.html.tera       # List all persons
│   ├── edit_person.html.tera  # Edit person form
│   └── roles.html.tera        # List all roles
├── Cargo.toml          # Rust dependencies
├── API.md              # REST API documentation
├── CLAUDE.md           # Development guidelines
└── README.md           # This file
```

## Environment Variables

The application requires a `DATABASE_URL` environment variable. Create a `.env` file:

```
DATABASE_URL=sqlite:./deleg.db
```

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

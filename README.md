# Deleg

A simple web application for managing people and their roles, built with Rust, Rocket, and htmx.

## Features

- View all persons in a clean table interface
- Add, edit, and delete person records
- RESTful API for person management
- Server-side rendering with Tera templates
- Dynamic interactions with htmx
- SQLite database with sqlx for type-safe queries
- Comprehensive test coverage

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

- `GET /people` - View all persons in a table
- `GET /people/:id` - Edit a specific person

### API Endpoints

- `GET /api/persons` - Get all persons (JSON)
- `GET /api/persons/:id` - Get a specific person (JSON)
- `POST /api/persons` - Create a new person (JSON)
- `PUT /api/persons/:id` - Update a person (JSON)
- `DELETE /api/persons/:id` - Delete a person

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
│   ├── models.rs       # Data models (Person)
│   ├── db.rs           # Database operations (CRUD)
│   └── routes.rs       # HTTP routes and handlers
├── templates/          # Tera HTML templates
│   ├── people.html.tera
│   └── edit_person.html.tera
├── Cargo.toml          # Rust dependencies
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

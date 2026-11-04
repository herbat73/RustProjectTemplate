# Rust Project Template

Modified Full stack rust template from https://github.com/sollambert/rust-full-stack-with-auth-template 

This template is built using Dioxus, Tailwinds, Axum, Sqlx, and Tauri.

This project is built to make full stack development with user authentication easily accessible for developers within an entirely Rust ecosystem.

In here you'll find some quick demos to get you started with an entirely Rust web environment!

Authentication/authorization is achieved with JSON Web Tokens. This is achieved in a two-part authentication process that involves the user signing in/creating an account at which point they are issued an authentication token for a defined amount of time.

With this token, users are then able to request authorization tokens to perform authorized actions depending on access level. Through frontend middleware, this process is handled automatically and the authorization token is placed into request headers as a Bearer auth token whenever a request is sent to the backend.

Web socket authentication functions slightly differently, as the authentication handshake occurs through the first message of a freshly opened websocket instead of being sent as a Bearer auth header.

The specific flavor of SQL is inferred from DATABASE_URL environment variable, however this package does allow for conditionally compiling with explicit support for SQLite and Postgres through their respective features if you would like to use flavor-specific syntax in constructed queries.

## Crates

- `frontend`: Dioxus frontend app for desktop client.
- `backend`: Axum backend restful and websocket api for desktop client.
- `server`: Axum server side restful and websocket api.
- `types`: Common types shared by frontend/backend/server.
- `tauri`: Tauri app for desktop client.

## Development

Install

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cargo install tauri-cli
cargo install sqlx-cli
```

Run desktop client app

```bash
cargo tauri dev
```

Run server side

```bash
cargo run --bin server
```

Bundle desktop client app

```bash
cargo tauri build
```

Run web frontend
```bash
cd crates/frontend
trunk serve
```

Run migrations (creates the database first if it does not exist)
```bash
# Postgres
scripts/migrate.sh postgres
# SQLite
sqlx database create 
scripts/migrate.sh sqlite
```

Revert migrations
```bash
# Postgres
sqlx migrate revert --source migrations/postgres
# SQLite
sqlx migrate revert --source migrations/sqlite
```

## Environment Variables

Required environment variables, these can be stored in a .env file at the top level of the repository if not set as OS environment variables are in .env.example file.
Copy as .env file and modify as you need.

## Contribute

Feel free to take a look at the current issues in this repo for anything that currently needs to be worked on.

You are also welcome to open a PR or a new issue if you see something is missing or could be improved upon.

## License

MIT

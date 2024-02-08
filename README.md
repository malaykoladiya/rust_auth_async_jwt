# rust_auth_async_jwt

## My Rust Project - Authentication using JWT from Auth0

This is the main entry point for the Rust project, illustrating how to set up a web server using Actix-web with Diesel for ORM and database pooling, and how to structure authentication and error handling.

### Features
- Actix-web for the web server and middleware support.
- Diesel for ORM and database operations.
- Actix-web-httpauth for authentication middleware.
- Environment configuration from `.env` file.
- Modular architecture with separate modules for authentication, errors, handlers, models, schema, and utilities.


### Modules

#### Handlers Module

This module contains the request handlers for user operations such as signing up, logging in,
and accessing the home page. It utilizes Actix Web for handling web requests and Diesel for database operations.


#### Errors Module
This module defines custom error types for the application. These errors encompass various failure states that
might occur during the operation of the application, such as database errors, connection pool errors, and
application-specific errors like token validation failures or internal server errors.
It leverages `thiserror` for defining error types in a way that is compatible with Rust's error handling paradigm.


#### Models Module
This module defines the data structures used throughout the application for interacting with the database.
It includes the following:
    - `User`: Struct for querying existing users from the database.
    - `NewUser`: Struct for inserting new users into the database.
    - `LoginCredentials`: Struct for handling login requests.


#### Errors Module
This module defines custom error types for the application's error handling logic.
The `ServiceError` enum is a central part of the error handling architecture, providing
a consistent interface for converting application errors into user-friendly HTTP responses.
This module ensures that different kinds of errors from the backend are translated into
appropriate HTTP status codes and messages.


#### Utility Functions Module
This module provides utility functions for password handling, including hashing and verifying passwords.
It leverages the `argonautica` crate to utilize the Argon2 algorithm for password security, which is
considered one of the most secure algorithms for this purpose. The functions here are essential for
user authentication processes, ensuring that passwords are stored and verified securely.



### Environment Configuration
The application configuration is managed through environment variables set in a `.env` file at the root of the project.

#### Setting Up the `.env` File
1. Create a new file named `.env` in the root directory of the project.
2. Add the necessary environment variables with your specific values:
    - DATABASE_URL=postgres://username:password@localhost/my_database
    - AUTH0_DOMAIN=your-auth0-domain
    - AUTH0_CLIENT_ID=your-auth0-client-id
    - AUTH0_CLIENT_SECRET=your-auth0-client-secret
    - AUTH0_AUDIENCE=your-auth0-audience
    - SERVER_ADDRESS=127.0.0.1:8080
    - SECRET_KEY=your-secret-key

Be sure to replace the placeholders with your actual settings.

**Important**: The `.env` file should never be committed to version control. Ensure it is included in your `.gitignore`.

##### Required Variables
- `DATABASE_URL`: Connection string for the PostgreSQL database.
- `AUTH0_*`: Configuration parameters for Auth0 integration.
- `SERVER_ADDRESS`: Address and port for the server to listen on.
- `SECRET_KEY`: A secret key used for securing operations like hashing.

For detailed setup instructions, refer to the [Auth0 documentation](https://auth0.com/docs).



### Getting Started


#### Prerequisites

Before you can run the server and interact with the database, you need to set up Diesel CLI and run migrations:
- Install Diesel CLI with `cargo install diesel_cli`.
- Note: Diesel CLI requires the appropriate database backend libraries to be installed on your system. For PostgreSQL, you'll need the `libpq` library.
- Ensure you have a running instance of PostgreSQL and have created the necessary database that matches your `DATABASE_URL` in the `.env` file.

#### Database Setup with Diesel
Once you have installed Diesel CLI and set up your database, you can run migrations to create the necessary tables and schema:
1. From the terminal, navigate to the root directory of the project.
2. Run `diesel setup` to set up the database specified in your `.env` file.
3. Run `diesel migration run` to apply migrations to your database.

#### Running Migrations
Whenever you change your database schema, you will create a new migration:
1. To create a new migration, run `diesel migration generate <migration_name>`.
2. This will create a new directory under `migrations/` with `up.sql` and `down.sql` files.
3. Write your SQL to alter the schema in the `up.sql` file and to revert your changes in the `down.sql` file.
4. Run `diesel migration run` to apply your new migrations to the database.
5. You can undo the last migration with `diesel migration revert`.

#### Launching the Application
After setting up the database, you can launch the application server:
1. Ensure you have Rust and Cargo installed.
2. Set up the `.env` file with your database URL and other environment-specific configurations.
3. Build the project with `cargo build`.
4. Run the server with `cargo run`.
The server will start and listen on the address and port specified in the `SERVER_ADDRESS` environment variable. You can now interact with the API endpoints defined in the handlers module.

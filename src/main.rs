//! # My Rust Project - Authentication using JWT from Auth0
//!
//! This is the main entry point for the Rust project, illustrating how to set up a web server using Actix-web with Diesel for ORM and database pooling, and how to structure authentication and error handling.
//!
//! ## Features
//! - Actix-web for the web server and middleware support.
//! - Diesel for ORM and database operations.
//! - Actix-web-httpauth for authentication middleware.
//! - Environment configuration from `.env` file.
//! - Modular architecture with separate modules for authentication, errors, handlers, models, schema, and utilities.
//!
//!
//! ## Modules
//!
//! ### Handlers Module
//!
//! This module contains the request handlers for user operations such as signing up, logging in,
//! and accessing the home page. It utilizes Actix Web for handling web requests and Diesel for database operations.
//!
//!
//! ### Errors Module
//! This module defines custom error types for the application. These errors encompass various failure states that
//! might occur during the operation of the application, such as database errors, connection pool errors, and
//! application-specific errors like token validation failures or internal server errors.
//! It leverages `thiserror` for defining error types in a way that is compatible with Rust's error handling paradigm.
//!
//!
//! ### Models Module
//! This module defines the data structures used throughout the application for interacting with the database.
//! It includes the following:
//!     - `User`: Struct for querying existing users from the database.
//!     - `NewUser`: Struct for inserting new users into the database.
//!     - `LoginCredentials`: Struct for handling login requests.
//!
//!
//! ### Errors Module
//! This module defines custom error types for the application's error handling logic.
//! The `ServiceError` enum is a central part of the error handling architecture, providing
//! a consistent interface for converting application errors into user-friendly HTTP responses.
//! This module ensures that different kinds of errors from the backend are translated into
//! appropriate HTTP status codes and messages.
//!
//!
//! ### Utility Functions Module
//! This module provides utility functions for password handling, including hashing and verifying passwords.
//! It leverages the `argonautica` crate to utilize the Argon2 algorithm for password security, which is
//! considered one of the most secure algorithms for this purpose. The functions here are essential for
//! user authentication processes, ensuring that passwords are stored and verified securely.
//!
//!
//!
//! ## Environment Configuration
//! The application configuration is managed through environment variables set in a `.env` file at the root of the project.
//!
//! ### Setting Up the `.env` File
//! 1. Create a new file named `.env` in the root directory of the project.
//! 2. Add the necessary environment variables with your specific values:
//!     - RUST_LOG=debug
//!     - RUN_MODE=development
//!     - DATABASE_URL=postgres://username:password@localhost/my_database
//!     - AUTH0_DOMAIN=your-auth0-domain
//!     - AUTH0_CLIENT_ID=your-auth0-client-id
//!     - AUTH0_CLIENT_SECRET=your-auth0-client-secret
//!     - AUTH0_AUDIENCE=your-auth0-audience
//!     - SERVER_ADDRESS=127.0.0.1:8080
//!     - SECRET_KEY=your-secret-key
//!
//! Be sure to replace the placeholders with your actual settings.
//!
//! **Important**: The `.env` file should never be committed to version control. Ensure it is included in your `.gitignore`.
//!
//! #### Required Variables
//! - `DATABASE_URL`: Connection string for the PostgreSQL database.
//! - `AUTH0_*`: Configuration parameters for Auth0 integration.
//! - `SERVER_ADDRESS`: Address and port for the server to listen on.
//! - `SECRET_KEY`: A secret key used for securing operations like hashing.
//!
//! For detailed setup instructions, refer to the [Auth0 documentation](https://auth0.com/docs).
//!
//!
//!
//! ## Getting Started
//!
//! 
//! ### Prerequisites
//! 
//! Before you can run the server and interact with the database, you need to set up Diesel CLI and run migrations:
//! - Install Diesel CLI with `cargo install diesel_cli`.
//! - Note: Diesel CLI requires the appropriate database backend libraries to be installed on your system. For PostgreSQL, you'll need the `libpq` library.
//! - Ensure you have a running instance of PostgreSQL and have created the necessary database that matches your `DATABASE_URL` in the `.env` file.
//! 
//! ### Database Setup with Diesel
//! Once you have installed Diesel CLI and set up your database, you can run migrations to create the necessary tables and schema:
//! 1. From the terminal, navigate to the root directory of the project.
//! 2. Run `diesel setup` to set up the database specified in your `.env` file.
//! 3. Run `diesel migration run` to apply migrations to your database.
//! 
//! ### Running Migrations
//! Whenever you change your database schema, you will create a new migration:
//! 1. To create a new migration, run `diesel migration generate <migration_name>`.
//! 2. This will create a new directory under `migrations/` with `up.sql` and `down.sql` files.
//! 3. Write your SQL to alter the schema in the `up.sql` file and to revert your changes in the `down.sql` file.
//! 4. Run `diesel migration run` to apply your new migrations to the database.
//! 5. You can undo the last migration with `diesel migration revert`.
//! 
//! ### Launching the Application
//! After setting up the database, you can launch the application server:
//! 1. Ensure you have Rust and Cargo installed.
//! 2. Set up the `.env` file with your database URL and other environment-specific configurations.
//! 3. Build the project with `cargo build`.
//! 4. Run the server with `cargo run`.
//! The server will start and listen on the address and port specified in the `SERVER_ADDRESS` environment variable. You can now interact with the API endpoints defined in the handlers module.


#[macro_use]
extern crate diesel; // ORM library for Rust

// dependencies
// Core Actix web functionalities, middleware support, HTTP server
use actix_web::{
    dev::ServiceRequest, middleware::Logger, web, web::Data, App, Error, HttpResponse, HttpServer,
};

// Authentication middleware for bearer tokens
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::middleware::HttpAuthentication;

// Diesel for database operations and connection pooling
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use env_logger::Env;
use log::{debug, error, info, warn};
use std::env;

// Modularization of the app into different components
mod auth; // Handles authentication logic
mod errors; // Custom error handling
mod handlers; // Request handlers for different routes
mod models; // Structs for database models
mod schema; // Generated database schema
mod utils; // Utility functions and common helpers

/// Type alias for using the database pool across the app
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Entry point of the application.
///
/// This function configures and starts the HTTP server, sets up database connection pooling,
/// and initializes the web application routes and middleware.
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok().expect("Failed to read .env file");

    // Initialize logger
    let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".to_string());
    env_logger::Builder::from_env(Env::default().default_filter_or(match run_mode.as_str() {
        "production" => "info",
        _ => "debug", // Default to debug for development or any other unspecified mode
    }))
    .init();

    info!("Application version: {}", env!("CARGO_PKG_VERSION"));
    // Log the current run mode
    info!("Running in {} mode", run_mode);

    // Database and server configuration
    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(e) => {
            error!("DATABASE_URL is not set: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "DATABASE_URL not set",
            ));
        }
    };
    let server_address =
        env::var("SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let manager: ConnectionManager<PgConnection> =
        ConnectionManager::<PgConnection>::new(&database_url);
    let pool: Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    // Example of adjusting configuration based on run mode
    if run_mode == "development" {
        debug!("Development-specific configuration applied");
    } else if run_mode == "production" {
        info!("Production-specific configuration applied");
    }

    // Setting up the HTTP server
    info!("Server will bind to {}", &server_address);
    HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(validator); // Authentication middleware setup
        App::new()
            .wrap(Logger::default()) // Log all requests
            .app_data(Data::new(pool.clone())) // Pass database pool to app
            .route("/users/signup", web::post().to(handlers::sign_up)) // Signup route
            .route("/users/login", web::post().to(handlers::login)) // Login route
            .service(
                web::scope("/users") // Scope for user-related routes
                    .wrap(auth) // Apply authentication middleware to all routes in this scope
                    .route("/homepage", web::get().to(handlers::home_page)), // Homepage route
            )
            .default_service(web::route().to(HttpResponse::NotFound)) // Default service for unmatched routes
    })
    .bind(server_address)? // Bind server to the specified address
    .run() // Start the server
    .await
}

/// Validator function to check the validity of JWT tokens in incoming requests.
///
/// This async function examines the bearer token provided in incoming HTTP requests,
/// validating them using the custom logic defined in the `auth` module. It ensures that
/// each request to secured endpoints has a valid authentication token.
async fn validator(
    req: ServiceRequest,     // Incoming request to validate
    credentials: BearerAuth, // Extracted bearer token from the request
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    debug!("Received token"); // Use debug for sensitive information

    // Extract the configuration or use default if not set
    let config = req
        .app_data::<Config>()
        .cloned()
        .unwrap_or_else(Config::default);

    // Validate the token asynchronously
    match auth::validate_token(credentials.token()).await {
        Ok(res) if res => {
            // Token is valid, proceed with the request
            info!("Token validated successfully for request: {:?}", req.path()); // Log successful validation
            Ok(req)
        }
        Ok(_) => {
            // Token is invalid, return an error response
            warn!("Invalid token received for request: {:?}", req.path()); // Use warn for invalid tokens
            Err((AuthenticationError::from(config).into(), req))
        }
        Err(e) => {
            // Error occurred during token validation, return an error response
            error!(
                "Error during token validation for request: {:?}: {:?}",
                req.path(),
                e
            ); // Log errors with context
            Err((AuthenticationError::from(config).into(), req))
        }
    }
}

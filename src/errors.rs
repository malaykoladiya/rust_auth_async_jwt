//! # Errors Module
//!
//! This module defines custom error types for the application. These errors encompass various failure states that
//! might occur during the operation of the application, such as database errors, connection pool errors, and
//! application-specific errors like token validation failures or internal server errors.
//!
//! It leverages `thiserror` for defining error types in a way that is compatible with Rust's error handling paradigm.

// Import necessary modules from Actix Web and Diesel for error handling and HTTP responses.
use actix_web::error::BlockingError;
use actix_web::{error::ResponseError, HttpResponse};
use diesel::result::Error as DieselError;
use r2d2::Error as R2d2Error;
use thiserror::Error; // Facilitates easy definition of error enums.

// Define a comprehensive enum for various service errors that might occur within the application.
#[derive(Error, Debug)]
pub enum ServiceError {
    // Represents generic internal server errors.
    #[error("Internal Server Error")]
    InternalServerError,

    #[error("Unauthorized")]
    Unauthorized,

    // Represents client-side input errors with a dynamic message.
    #[error("BadRequest: {0}")]
    BadRequest(String),

    // Represents errors related to environment configuration issues.
    #[error("Environment Error")]
    EnvironmentError,

    // Error for when JWKS (JSON Web Key Set) fetching fails, which is critical for JWT validation.
    #[error("Could not fetch JWKS")]
    JWKSFetchError,

    // Error for when token validation fails, indicating the JWT is invalid or expired.
    #[error("Token Validation Error")]
    TokenValidationError,

    #[error("The requested resource was not found")]
    NotFound,

    // Integrates Diesel database errors into the service error types.
    #[error("Database error: {0}")]
    Diesel(#[from] DieselError),

    // Integrates r2d2 connection pool errors into the service error types.
    #[error("Connection pool error: {0}")]
    Pool(#[from] R2d2Error),
}

// Implements conversion from Actix Web's BlockingError to ServiceError.
impl From<BlockingError> for ServiceError {
    fn from(_e: BlockingError) -> Self {
        ServiceError::InternalServerError
    }
}

// Implement how service errors are converted into HTTP responses.
impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            // Each variant maps to an appropriate HTTP response.
            ServiceError::InternalServerError => HttpResponse::InternalServerError()
                .json("Internal Server Error. Please try again later."),
            ServiceError::BadRequest(message) => HttpResponse::BadRequest().json(message),
            ServiceError::EnvironmentError => HttpResponse::InternalServerError()
                .json("Configuration error. Please check server configurations."),
            ServiceError::JWKSFetchError => HttpResponse::InternalServerError()
                .json("Failed to fetch JWKS. Please check JWKS endpoint."),
            ServiceError::TokenValidationError => {
                HttpResponse::Unauthorized().json("Invalid token. Token validation failed.")
            }
            ServiceError::Diesel(_) => HttpResponse::InternalServerError()
                .json("Database operation failed. Please try again later."),
            ServiceError::Pool(_) => {
                HttpResponse::InternalServerError().json("Database connection pool error.")
            }
            ServiceError::NotFound => {
                HttpResponse::NotFound().json("The requested resource was not found.")
            }
            ServiceError::Unauthorized => {
                HttpResponse::Unauthorized().json("Invalid credentials or password")
            }
        }
    }
}

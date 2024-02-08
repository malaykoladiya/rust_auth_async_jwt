//! # Handlers Module
//!
//! This module contains the request handlers for user operations such as signing up, logging in,
//! and accessing the home page. It utilizes Actix Web for handling web requests and Diesel for database operations.

/// Dependencies
/// Importing necessary modules and structs for handling database operations, web requests, and authentication.
use super::models::{LoginCredentials, NewUser, User};
use super::schema::users::dsl::*;
use super::Pool;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use actix_web::{web, HttpResponse, Responder, Result as ActixResult};
use diesel::dsl::insert_into;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use crate::auth::request_auth0_token;
use crate::diesel::ExpressionMethods;
use crate::errors::ServiceError;
use crate::utils::{hash_password, verify_password};
use diesel::OptionalExtension;

/// Struct for user input on sign-up.
#[derive(Debug, Serialize, Deserialize)]
pub struct InputUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub user_password: String,
}

/// Handler for processing user sign-up requests.
///
/// This asynchronous function takes a database connection pool and user input data,
/// validates the input, hashes the password, and inserts the new user into the database.
///
/// # Arguments
///
/// * `db`: Database connection pool.
/// * `item`: User input data.
///
/// # Returns
///
/// This function returns an Actix result with either an HTTP response indicating success or a ServiceError.

pub async fn sign_up(
    db: web::Data<Pool>,        // Database connection pool
    item: web::Json<InputUser>, // User input data
) -> ActixResult<HttpResponse, ServiceError> {
    // Validate input fields are not empty.
    if item.first_name.is_empty()
        || item.last_name.is_empty()
        || item.email.is_empty()
        || item.user_password.is_empty()
    {
        warn!("Signup failed: All fields are required.");
        return Err(ServiceError::BadRequest(
            "Invalid input: All fields are required".to_string(),
        ));
    }

    // Hash the user's password for secure storage.
    let hashed_password = hash_password(&item.user_password)
        .await
        .map_err(|_| ServiceError::BadRequest("Password hashing failed".to_string()))?;

    let mut input_user = item.into_inner();
    input_user.user_password = hashed_password; // Update the input user with the hashed password.

    // Insert the new user into the database.
    let user_result = web::block(move || {
        let mut conn = db.get().map_err(ServiceError::Pool)?;

        let new_user = NewUser {
            first_name: input_user.first_name,
            last_name: input_user.last_name,
            email: input_user.email,
            user_password: input_user.user_password, // Use the hashed password here
            created_at: chrono::Local::now().naive_local(),
        };
        insert_into(users)
            .values(&new_user)
            .get_result::<User>(&mut conn)
            .map_err(ServiceError::Diesel)
    })
    .await
    .map_err(|e: actix_web::error::BlockingError| ServiceError::from(e))?;

    // Return the created user or an error.
    match user_result {
        Ok(user) => {
            info!("New user created with email: {}", user.email);
            Ok(HttpResponse::Created().json(user))
        }
        Err(e) => {
            error!("Failed to create user: {:?}", e);
            Err(e.into()) // Convert to ServiceError if not already
        }
    }
}

/// Handler for processing user login requests.
///
/// This asynchronous function authenticates a user by their email and password.
/// If authentication succeeds, it requests and returns an Auth0 token.
///
/// # Arguments
///
/// * `db`: Database connection pool.
/// * `credentials`: User's login credentials.
///
/// # Returns
///
/// This function returns an Actix result with either an HTTP response containing the Auth0 token or an ActixError.

pub async fn login(
    db: web::Data<Pool>,                      // Database connection pool
    credentials: web::Json<LoginCredentials>, // User's login credentials
) -> ActixResult<HttpResponse, ServiceError> {
    debug!("Attempting login for user: {}", credentials.email);

    let user_email = credentials.email.clone();
    let password = credentials.password.clone();

    // Attempt to find the user by email.
    let user_data = web::block(move || find_user_by_email(db, &user_email))
        .await
        .map_err(|e| ServiceError::from(e))?;

    // If a user is found, verify their password.
    if let Ok(Some(user_data)) = user_data {
        let verification_result = verify_password(&password, &user_data.user_password);

        // If password verification is successful, request an Auth0 token.
        match verification_result {
            Ok(true) => {
                // Fetch the JWT token from Auth0
                match request_auth0_token().await {
                    Ok(auth0_response) => {
                        // Send the Auth0 token back to the user
                        // You might want to create a new type for this response
                        info!("Auth0 token received for user: {}", &credentials.email);
                        Ok(HttpResponse::Ok().json(auth0_response))
                    }
                    Err(e) => {
                        // Handle the error, possibly returning a ServiceError
                        error!("Error fetching token from Auth0: {:?}", e);
                        Err(ServiceError::JWKSFetchError)
                    }
                }
            }
            Ok(false) => {
                warn!(
                    "Login failed for user: {}, invalid credentials.",
                    &credentials.email
                );
                Err(ServiceError::Unauthorized)
            }
            Err(_) => {
                warn!(
                    "Login failed for user: {}, password verification error.",
                    &credentials.email
                );
                Err(ServiceError::Unauthorized)
            }
        }
    } else {
        warn!(
            "Login failed for user: {}, user not found.",
            &credentials.email
        );
        Err(ServiceError::NotFound)
    }
}

/// Utility function to find a user by their email in the database.
///
/// # Arguments
///
/// * `pool`: Database connection pool.
/// * `user_email`: The email of the user to find.
///
/// # Returns
///
/// This function returns a Result with either an option containing the user or a Diesel error.
fn find_user_by_email(
    pool: web::Data<Pool>,
    user_email: &str,
) -> Result<Option<User>, ServiceError> {
    debug!("Looking for user by email: {}", user_email);

    let mut conn = pool.get().map_err(ServiceError::Pool)?;
    let user = users
        .filter(email.eq(user_email))
        .first::<User>(&mut conn)
        .optional()
        .map_err(ServiceError::Diesel)?;

    // If user is None, return NotFound error
    match user {
        Some(user) => Ok(Some(user)),
        None => Err(ServiceError::NotFound),
    }
}

/// Handler to display the home page.
///
/// This function is accessible only to authenticated users and returns a simple welcome message.
///
/// # Returns
///
/// This function returns an Actix web response with the home page content.
pub async fn home_page() -> impl Responder {
    info!("Home page accessed.");
    HttpResponse::Ok().body("Welcome to HomePage!")
}

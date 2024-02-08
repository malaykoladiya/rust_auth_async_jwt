//! # Errors Module
//!
//! This module defines custom error types for the application's error handling logic.
//! The `ServiceError` enum is a central part of the error handling architecture, providing
//! a consistent interface for converting application errors into user-friendly HTTP responses.
//! This module ensures that different kinds of errors from the backend are translated into
//! appropriate HTTP status codes and messages.

// Import relevant crates and modules for handling JWTs, serialization, and environment variables
use crate::errors::ServiceError;
use alcoholic_jwt::{token_kid, validate, Validation, JWKS};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

// Claims struct used for deserializing JWT claims
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

// Represents the request payload for obtaining a token from Auth0
#[derive(Serialize)]
pub struct Auth0TokenRequest {
    client_id: String,
    client_secret: String,
    audience: String,
    grant_type: String,
}

// Represents the response from Auth0 upon successful token request
#[derive(Serialize, Deserialize)]
pub struct Auth0TokenResponse {
    access_token: String,
    token_type: String,
}

// Asynchronously requests a JWT token from Auth0
pub async fn request_auth0_token() -> Result<Auth0TokenResponse, Box<dyn std::error::Error>> {
    // Logging the attempt to request a token
    info!(
        "Requesting Auth0 token for audience: {}",
        env::var("AUTH0_AUDIENCE").unwrap_or_default()
    );

    let client = reqwest::Client::new();

    // Environment variables should be used to avoid hardcoding sensitive information
    let client_id = env::var("AUTH0_CLIENT_ID")?;
    let client_secret = env::var("AUTH0_CLIENT_SECRET")?;
    let audience = env::var("AUTH0_AUDIENCE")?; // The identifier of your API in Auth0
    let auth0_domain = env::var("AUTH0_DOMAIN")?;
    let auth0_url = format!("https://{}/oauth/token", auth0_domain);

    // Prepare the request payload
    let token_request = Auth0TokenRequest {
        client_id,
        client_secret,
        audience,
        grant_type: "client_credentials".to_string(),
    };

    // Send the request to Auth0 and await the response
    let response = client.post(&auth0_url).json(&token_request).send().await?;

    match response.json::<Auth0TokenResponse>().await {
        Ok(token_response) => {
            info!("Successfully received Auth0 token");
            // Return the token response
            Ok(token_response)
        }
        Err(e) => {
            error!("Failed to receive Auth0 token: {:?}", e);
            Err(e.into())
        }
    }
}


// Validates a JWT token using JWKS from a specified authority
pub async fn validate_token(token: &str) -> Result<bool, ServiceError> {
    debug!("Validating JWT token");

    let authority = env::var("AUTHORITY").map_err(|_| ServiceError::EnvironmentError)?;
    let jwks_uri = format!("{}{}", authority, ".well-known/jwks.json");

    // Fetch the JSON Web Key Set (JWKS) from the authority
    let jwks = fetch_jwks(&jwks_uri).await.map_err(|e| {
        error!("Error fetching JWKS: {:?}", e);
        ServiceError::JWKSFetchError
    })?;

    // Prepare validation criteria
    let validations = vec![Validation::Issuer(authority), Validation::SubjectPresent];
    let kid = match token_kid(&token) {
        Ok(res) => res.expect("failed to decode kid"),
        Err(_) => return Err(ServiceError::JWKSFetchError),
    };

    // Find the corresponding JWK in the JWKS for the token's KID
    let jwk = jwks.find(&kid).ok_or(ServiceError::JWKSFetchError)?;
    let res = validate(token, jwk, validations)
        .map(|_| true)
        .map_err(|_| ServiceError::TokenValidationError);

    // Return true if token is valid, false otherwise
    match res {
        Ok(_) => {
            info!("JWT token validated successfully");
            Ok(true)
        }
        Err(e) => {
            warn!("JWT token validation failed: {:?}", e);
            Err(e)
        }
    }
}

// Asynchronously fetches JWKS from a specified URI
async fn fetch_jwks(uri: &str) -> Result<JWKS, Box<dyn Error>> {
    // Perform the HTTP GET request
    let res = match reqwest::get(uri).await {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to fetch JWKS: {:?}", e);
            return Err(e.into());
        }
    };
    let body = match res.text().await {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to read JWKS response body: {:?}", e);
            return Err(e.into());
        }
    };

    // Deserialize the JWKS from the response body
    match serde_json::from_str(&body) {
        Ok(val) => {
            debug!("Successfully fetched and deserialized JWKS ");
            Ok(val)
        }
        Err(e) => {
            error!("Failed to deserialize JWKS: {:?}", e);
            Err(e.into())
        }
    }
}

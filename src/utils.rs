//! # Utility Functions Module
//! This module provides utility functions for password handling, including hashing and verifying passwords.
//! It leverages the `argonautica` crate to utilize the Argon2 algorithm for password security, which is
//! considered one of the most secure algorithms for this purpose. The functions here are essential for
//! user authentication processes, ensuring that passwords are stored and verified securely.

// Import argonautica crate for hashing and verifying passwords.
use argonautica::Hasher;
use argonautica::Verifier;
use std::env;

/// Hashes a password using the Argon2 algorithm.
///
/// This function takes a plaintext password as input and returns the hashed password.
/// It retrieves the secret key from the environment variables to use in the hashing process.
/// The Argon2 algorithm is considered one of the most secure hashing algorithms for passwords.
///
/// # Arguments
///
/// * `password` - A string slice that holds the password to be hashed.
///
/// # Returns
///
/// This function returns a `Result` which is Ok containing the hashed password as a `String`
/// if the operation is successful, or an `argonautica::Error` if it fails.

pub async fn hash_password(password: &str) -> Result<String, argonautica::Error> {
    // Retrieve the secret key from environment variable.
    let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY must be set");

    // Initialize the hasher with default parameters.
    let mut hasher = Hasher::default();

    // Set the password, secret key, and perform the hashing.
    hasher
        .with_password(password)
        .with_secret_key(secret_key)
        .hash() // Perform the hash operation and return the result.
}

/// Verifies a password against a hash.
///
/// This function is used to verify if a given plaintext password matches the hashed version.
/// It is primarily used during the login process to authenticate users.
///
/// # Arguments
///
/// * `password` - A string slice that holds the plaintext password to verify.
/// * `hash` - A string slice that holds the hashed password to compare against.
///
/// # Returns
///
/// Returns a `Result` which is Ok containing a boolean value `true` if the password matches the hash,
/// or `false` otherwise. It may also return an `argonautica::Error` if the verification process fails.

pub fn verify_password(password: &str, hash: &str) -> Result<bool, argonautica::Error> {
    // Retrieve the secret key from environment variable.
    let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY must be set");

    // Initialize the verifier with default parameters.
    let mut verifier = Verifier::default();

    // Set the hash, password, secret key, and perform the verification.
    verifier
        .with_hash(hash)
        .with_password(password)
        .with_secret_key(secret_key)
        .verify() // Perform the verification and return the result.
}

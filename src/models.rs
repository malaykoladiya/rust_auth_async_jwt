//! # Models Module
//!
//! This module defines the data structures used throughout the application for interacting with the database.
//! It includes the following:
//! - `User`: Struct for querying existing users from the database.
//! - `NewUser`: Struct for inserting new users into the database.
//! - `LoginCredentials`: Struct for handling login requests.

// Import necessary crates and modules for ORM and serialization.
use crate::schema::*;
use serde::{Deserialize, Serialize};

// User struct for querying existing users from the database.
// It implements Serialize and Deserialize for easy conversion between JSON and Rust structs.
#[derive(Serialize, Debug, Queryable, Deserialize)]
pub struct User {
    pub id: i32,                           // Unique identifier for the user.
    pub first_name: String,                // User's first name.
    pub last_name: String,                 // User's last name.
    pub email: String,                     // User's email address.
    pub user_password: String,             // Hashed password for the user.
    pub created_at: chrono::NaiveDateTime, // Timestamp of user creation.
}

// NewUser struct for inserting new users into the database.
// It is marked with Insertable to specify that it can be used in INSERT statements.
#[derive(Insertable, Debug)]
#[diesel(table_name = users)] // Specify the database table associated with this struct.
pub struct NewUser {
    pub first_name: String,                // User's first name.
    pub last_name: String,                 // User's last name.
    pub email: String,                     // User's email address.
    pub user_password: String,             // Hashed password for the user.
    pub created_at: chrono::NaiveDateTime, // Timestamp of user creation, set at the time of insertion.
}

// LoginCredentials struct for handling login requests.
// It includes fields for email and password as provided by the user during login attempts.
#[derive(Debug, Deserialize)]
pub struct LoginCredentials {
    pub email: String,    // Email provided by the user for login.
    pub password: String, // Password provided by the user for login.
}

use std::{fmt, str::FromStr};

use email_address::EmailAddress;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RegisterUser {
    pub username: String,
    pub pass: String,
    pub email: String,
}

impl fmt::Display for RegisterUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Username: {}\nPass: {}\nEmail: {}",
            self.username, self.pass, self.email
        )
    }
}

impl RegisterUser {
    pub fn update_field(&self, key: &str, value: String) -> Result<Self, String> {
        let mut new_self = self.clone();
        match key {
            "username" => new_self.username = value,
            "pass" => new_self.pass = value,
            "email" => new_self.email = value,
            _ => return Err(format!("Key not found: {}", key)),
        }
        Ok(new_self)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LoginUser {
    pub username: String,
    pub pass: String,
}

impl fmt::Display for LoginUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Username: {}\nPass: {}", self.username, self.pass)
    }
}

impl LoginUser {
    pub fn update_field(&self, key: &str, value: String) -> Result<Self, String> {
        let mut new_self = self.clone();
        match key {
            "username" => new_self.username = value,
            "pass" => new_self.pass = value,
            _ => return Err(format!("Key not found: {}", key)),
        }
        Ok(new_self)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ResetUser {
    #[cfg_attr(feature = "openapi", schema(value_type = String, format = "email", example = "user@example.com"))]
    pub email_address: EmailAddress,
    pub pass: String,
}

impl ResetUser {
    pub fn new(&self) -> ResetUser {
        Self {
            email_address: EmailAddress::new_unchecked(""),
            pass: String::new(),
        }
    }
    pub fn update_field(&self, key: &str, value: String) -> Result<Self, String> {
        let mut new_self = self.clone();
        match key {
            "pass" => new_self.pass = value,
            "email" => new_self.email_address = EmailAddress::from_str(&value).unwrap(),
            _ => return Err(format!("Key not found: {}", key)),
        }
        Ok(new_self)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UserInfo {
    pub uuid: String,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
}

impl fmt::Display for UserInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UUID: {}\nUsername: {}\nEmail: {}\nIs Admin: {}",
            self.uuid, self.username, self.email, self.is_admin
        )
    }
}

impl UserInfo {
    pub fn new() -> Self {
        Self {
            uuid: String::new(),
            username: String::new(),
            email: String::new(),
            is_admin: false,
        }
    }
}

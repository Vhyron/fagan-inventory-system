use crate::database;
use rusqlite::{params, Result as RusqliteResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use bcrypt;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

pub fn login(credentials: &LoginRequest) -> Result<AuthResponse, String> {
    let conn = database::establish_connection().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT id, username, password_hash, role, created_at, updated_at FROM users WHERE username = ?")
        .map_err(|e| e.to_string())?;
    
    let user_result = stmt.query_row(params![credentials.username], |row| {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password_hash: row.get(2)?,
            role: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    });
    
    match user_result {
        Ok(user) => {
            let password_matches = bcrypt::verify(&credentials.password, &user.password_hash)
                .map_err(|e| e.to_string())?;
            
            if password_matches {
                Ok(AuthResponse {
                    success: true,
                    message: "Login successful".to_string(),
                    user: Some(UserResponse {
                        id: user.id,
                        username: user.username,
                        role: user.role,
                    }),
                })
            } else {
                Ok(AuthResponse {
                    success: false,
                    message: "Invalid credentials".to_string(),
                    user: None,
                })
            }
        },
        Err(_) => {
            Ok(AuthResponse {
                success: false,
                message: "Invalid credentials".to_string(),
                user: None,
            })
        }
    }
}

pub fn create_secretary_user(request: &CreateUserRequest, creator_username: &str) -> Result<AuthResponse, String> {
    let conn = database::establish_connection().map_err(|e| e.to_string())?;
    
    // check if creator is an admin
    let mut stmt = conn.prepare("SELECT role FROM users WHERE username = ?")
        .map_err(|e| e.to_string())?;
    
    let creator_role: RusqliteResult<String> = stmt.query_row(params![creator_username], |row| row.get(0));
    
    match creator_role {
        Ok(role) => {
            if role != "admin" {
                return Ok(AuthResponse {
                    success: false,
                    message: "Only admins can create secretary accounts".to_string(),
                    user: None,
                });
            }
        },
        Err(_) => {
            return Ok(AuthResponse {
                success: false,
                message: "Creator not found".to_string(),
                user: None,
            });
        }
    }
    
    // create new secretary user
    let password_hash = bcrypt::hash(&request.password, 10).map_err(|e| e.to_string())?;
    let user_id = Uuid::new_v4().to_string();
    let now = chrono::Local::now().to_rfc3339();
    
    let result = conn.execute(
        "INSERT INTO users (id, username, password_hash, role, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            user_id,
            request.username,
            password_hash,
            "secretary",
            now,
            now,
        ],
    );
    
    match result {
        Ok(_) => {
            Ok(AuthResponse {
                success: true,
                message: "Secretary account created successfully".to_string(),
                user: Some(UserResponse {
                    id: user_id,
                    username: request.username.clone(),
                    role: "secretary".to_string(),
                }),
            })
        },
        Err(e) => {
            if e.to_string().contains("UNIQUE constraint failed") {
                Ok(AuthResponse {
                    success: false,
                    message: "Username already exists".to_string(),
                    user: None,
                })
            } else {
                Err(e.to_string())
            }
        }
    }
}

pub fn get_all_users() -> Result<Vec<UserResponse>, String> {
    let conn = database::establish_connection().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT id, username, role FROM users ORDER BY role, username")
        .map_err(|e| e.to_string())?;
    
    let users_iter = stmt.query_map([], |row| {
        Ok(UserResponse {
            id: row.get(0)?,
            username: row.get(1)?,
            role: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut users = Vec::new();
    for user in users_iter {
        users.push(user.map_err(|e| e.to_string())?);
    }
    
    Ok(users)
}

pub fn change_password(user_id: &str, old_password: &str, new_password: &str) -> Result<AuthResponse, String> {
    let conn = database::establish_connection().map_err(|e| e.to_string())?;
    
    // get current password hash
    let mut stmt = conn.prepare("SELECT password_hash FROM users WHERE id = ?")
        .map_err(|e| e.to_string())?;
    
    let current_hash: String = stmt.query_row(params![user_id], |row| row.get(0))
        .map_err(|_| "User not found".to_string())?;
    
    // verify old password
    let password_matches = bcrypt::verify(old_password, &current_hash)
        .map_err(|e| e.to_string())?;
    
    if !password_matches {
        return Ok(AuthResponse {
            success: false,
            message: "Current password is incorrect".to_string(),
            user: None,
        });
    }
    
    // hash and update new password
    let new_hash = bcrypt::hash(new_password, 10).map_err(|e| e.to_string())?;
    let now = chrono::Local::now().to_rfc3339();
    
    conn.execute(
        "UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?",
        params![new_hash, now, user_id],
    ).map_err(|e| e.to_string())?;
    
    Ok(AuthResponse {
        success: true,
        message: "Password changed successfully".to_string(),
        user: None,
    })
}

pub fn deactivate_secretary(user_id: &str, admin_username: &str) -> Result<AuthResponse, String> {
    let conn = database::establish_connection().map_err(|e| e.to_string())?;
    
    // check if requester is an admin
    let mut stmt = conn.prepare("SELECT role FROM users WHERE username = ?")
        .map_err(|e| e.to_string())?;
    
    let admin_role: RusqliteResult<String> = stmt.query_row(params![admin_username], |row| row.get(0));
    
    match admin_role {
        Ok(role) => {
            if role != "admin" {
                return Ok(AuthResponse {
                    success: false,
                    message: "Only admins can deactivate secretary accounts".to_string(),
                    user: None,
                });
            }
        },
        Err(_) => {
            return Ok(AuthResponse {
                success: false,
                message: "Admin not found".to_string(),
                user: None,
            });
        }
    }
    
    // check if target is a secretary
    let mut stmt = conn.prepare("SELECT role FROM users WHERE id = ?")
        .map_err(|e| e.to_string())?;
    
    let user_role: RusqliteResult<String> = stmt.query_row(params![user_id], |row| row.get(0));
    
    match user_role {
        Ok(role) => {
            if role != "secretary" {
                return Ok(AuthResponse {
                    success: false,
                    message: "Only secretary accounts can be deactivated".to_string(),
                    user: None,
                });
            }
        },
        Err(_) => {
            return Ok(AuthResponse {
                success: false,
                message: "User not found".to_string(),
                user: None,
            });
        }
    }
    
    // deactivate the secretary account
    conn.execute(
        "DELETE FROM users WHERE id = ? AND role = 'secretary'",
        params![user_id],
    ).map_err(|e| e.to_string())?;
    
    Ok(AuthResponse {
        success: true,
        message: "Secretary account deactivated successfully".to_string(),
        user: None,
    })
}
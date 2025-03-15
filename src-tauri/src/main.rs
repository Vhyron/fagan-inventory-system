// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod auth;

use auth::{LoginRequest, CreateUserRequest};

#[tauri::command]
fn login(credentials: LoginRequest) -> Result<auth::AuthResponse, String> {
    auth::login(&credentials)
}

#[tauri::command]
fn create_secretary(request: CreateUserRequest, admin_username: String) -> Result<auth::AuthResponse, String> {
    auth::create_secretary_user(&request, &admin_username)
}

#[tauri::command]
fn get_users() -> Result<Vec<auth::UserResponse>, String> {
    auth::get_all_users()
}

#[tauri::command]
fn change_password(user_id: String, old_password: String, new_password: String) -> Result<auth::AuthResponse, String> {
    auth::change_password(&user_id, &old_password, &new_password)
}

#[tauri::command]
fn deactivate_secretary(user_id: String, admin_username: String) -> Result<auth::AuthResponse, String> {
    auth::deactivate_secretary(&user_id, &admin_username)
}

fn main() {
    match database::init_database() {
        Ok(_) => {
            println!("Database initialized successfully");
        },
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    }    

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            login,
            create_secretary,
            get_users,
            change_password,
            deactivate_secretary
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
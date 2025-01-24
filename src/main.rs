use std::{
    fs::File, io::Write, sync::{Arc, Mutex}
};
use chrono::Utc;

use axum::{extract::State, http::StatusCode, response::Html, routing::get, Form, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

struct PersistentState {
    verify_list: Arc<Mutex<Vec<VerifiedUser>>>,
    civ_token: Arc<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct VerifyRequest {
    method: Option<String>,
    ckey: String,
    discord: String,
    token: String,
}

type VerifyFile = Vec<VerifiedUser>;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VerifiedUser {
    ss13: Option<String>,
    discord: Option<String>,
    create_time: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let v_file = load_verify_file()?;
    let (host_addr, civ_token) = load_env_config()?;
    let state = create_app_state(v_file, civ_token);
    let app = create_router(state);
    
    println!("Listening on: {}", host_addr);
    let listener = TcpListener::bind(&host_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn load_verify_file() -> Result<VerifyFile, Box<dyn std::error::Error>> {
    let v_file = File::open("verify.json")?;
    Ok(serde_json::from_reader(v_file)?)
}

fn load_env_config() -> Result<(String, String), Box<dyn std::error::Error>> {
    match File::open(".env") {
        Ok(_) => {
            dotenvy::dotenv()?;
            let addr = std::env::var("HOST_ADDR")?;
            let port = std::env::var("HOST_PORT")?;
            let civ_token = std::env::var("CIV_TOKEN")?;
            
            if civ_token == "changeme" && addr != "127.0.0.1" {
                return Err("Cannot use default token with non-localhost address!".into());
            }
            Ok((format!("{}:{}", addr, port), civ_token))
        }
        Err(_) => {
            let mut file = File::create(".env")?;
            println!("No .env file found. Creating one with default values.");
            file.write_all(b"HOST_ADDR=127.0.0.1\nHOST_PORT=8010\nCIV_TOKEN=changeme")?;
            Ok(("127.0.0.1:8010".to_string(), "changeme".to_string()))
        }
    }
}

fn create_app_state(v_file: VerifyFile, civ_token: String) -> Arc<PersistentState> {
    Arc::new(PersistentState {
        verify_list: Arc::new(Mutex::new(v_file)),
        civ_token: Arc::new(civ_token),
    })
}

fn create_router(state: Arc<PersistentState>) -> Router {
    Router::new()
        .route("/", get(|| async { 
            (StatusCode::FORBIDDEN, Html("<h1>Forbidden</h1>")) 
        }))
        .route("/verified", get(get_verified).post(update_verified))
        .with_state(state)
}

async fn get_verified(State(state): State<Arc<PersistentState>>) -> (StatusCode, Json<Vec<VerifiedUser>>) {
    let list = state.verify_list.lock().unwrap();
    println!("Got a request for data");
    (StatusCode::OK, Json(list.clone()))
}

fn write_json(file: &str, data: VerifyFile) {
    let file = match File::create(file) {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to open file: {}", e);
            return;
        }
    };
    match serde_json::to_writer(file, &data) {
        Ok(_) => (),
        Err(e) => println!("Failed to write to file: {}", e),
    }
}

async fn update_verified(
    State(state): State<Arc<PersistentState>>,
    Form(form): Form<VerifyRequest>,
) -> (StatusCode, &'static str) {
    println!("Request received: {:?}", form);
    
    if *state.civ_token != "changeme" && form.token != *state.civ_token {
        return (StatusCode::UNAUTHORIZED, "Invalid token");
    }

    let mut list = state.verify_list.lock().unwrap();
    let should_delete = form.method
        .as_ref()
        .map_or(false, |m| m.trim().to_lowercase() == "delete");

    let user_pos = list.iter().position(|user| {
        (user.discord.as_ref() == Some(&form.discord)) || 
        (user.ss13.as_ref() == Some(&form.ckey))
    });

    match (should_delete, user_pos) {
        (true, Some(pos)) => {
            list.remove(pos);
            write_json("verify.json", list.clone());
            (StatusCode::OK, "Deleted")
        },
        (true, None) => (StatusCode::FORBIDDEN, "User not found"), // This is a stupid use of HTTP 403. Blame Valithor.
        (false, Some(_)) => (StatusCode::FORBIDDEN, "User already exists"),
        (false, None) => {
            list.push(VerifiedUser {
                ss13: Some(form.ckey),
                discord: Some(form.discord),
                create_time: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
            });
            write_json("verify.json", list.clone());
            (StatusCode::OK, "Added user")
        }
    }
}

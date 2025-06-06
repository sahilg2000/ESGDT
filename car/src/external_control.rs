use bevy::prelude::*;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::task;
use tokio_tungstenite::accept_async;
use futures_util::stream::StreamExt;
use futures_util::SinkExt; 
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio::sync::Mutex as AsyncMutex;
use tokio::net::TcpStream;

use crate::control::CarControl;
use crate::control::user_control_system;

// Define a Resource to hold external data
#[derive(Resource)]
pub struct ExternalControls {
    control: Arc<Mutex<CarControl>>,
    last_update: Arc<Mutex<Instant>>, // Track the last time external input was received
}

pub struct ExternalControlPlugin;

impl Plugin for ExternalControlPlugin {
    fn build(&self, app: &mut App) {
        // Create a single Tokio runtime here
        let rt = Runtime::new().expect("Failed to create Tokio runtime");

        // Shared CarControl and last_update
        let control = Arc::new(Mutex::new(CarControl::default()));
        let last_update = Arc::new(Mutex::new(Instant::now()));

        // Clone these for server thread
        let websocket_control = control.clone();
        let websocket_last_update = last_update.clone();

        // Spawn a thread that uses THIS runtime
        thread::spawn(move || {
            rt.block_on(async move {
                start_websocket_server(websocket_control, websocket_last_update).await;
            });
        });

        // Insert resources into Bevy
        app.insert_resource(ExternalControls { control, last_update })
           .add_systems(Update, update_from_external_controls.after(user_control_system),);
    }
}

// Receive WebSocket messages and update CarControl
fn update_from_external_controls(
    external_controls: Res<ExternalControls>,
    mut car_control: ResMut<CarControl>,
) {
    // If no recent input in last 5s, skip
    if let Ok(last_update) = external_controls.last_update.lock() {
        if last_update.elapsed().as_secs() > 5 {
            return;
        }
    }

    // Copy from external to main CarControl if changed
    if let Ok(external) = external_controls.control.lock() {
        if car_control.throttle != external.throttle
            || car_control.brake != external.brake
            || car_control.steering != external.steering
        {
            car_control.throttle = external.throttle;
            car_control.brake = external.brake;
            car_control.steering = external.steering;

            println!(
                "Updated CarControl - Throttle: {:.2}, Brake: {:.2}, Steering: {:.2}",
                car_control.throttle, car_control.brake, car_control.steering
            );

        }
    }
}

// WebSocket Server to listen for external inputs
async fn start_websocket_server(
    control: Arc<Mutex<CarControl>>,
    last_update: Arc<Mutex<Instant>>,
) {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind WebSocket server on :8080");

    println!("Bevy WebSocket server listening on ws://127.0.0.1:8080");

    while let Ok((stream, _)) = listener.accept().await {
        println!("New WebSocket connection established.");
        match accept_async(stream).await {
            Ok(ws_stream) => {
                let c = control.clone();
                let lu = last_update.clone();
                task::spawn(handle_websocket_connection(ws_stream, c, lu));
            }
            Err(e) => {
                eprintln!("Failed to accept WebSocket connection: {}", e);
            }
        }
    }
}

// Handle incoming WebSocket messages
async fn handle_websocket_connection(
    mut ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    control: Arc<Mutex<CarControl>>,
    last_update: Arc<Mutex<Instant>>,
) {
    println!("Listening for incoming WebSocket messages...");

    while let Some(Ok(msg)) = ws_stream.next().await {
        println!("Received WebSocket message: {:?}", msg);

        if let Message::Text(text) = msg {
            if let Some((throttle, brake, steering)) = parse_control_data(&text) {
                println!(
                    "Parsed control data - Throttle: {:.2}, Brake: {:.2}, Steering: {:.2}",
                    throttle, brake, steering
                );
                if let Ok(mut car_control) = control.lock() {
                    car_control.throttle = throttle;
                    car_control.brake = brake;
                    car_control.steering = steering;
                    *last_update.lock().unwrap() = Instant::now();
                }
            } else {
                println!("Failed to parse control data from: {}", text);
            }
        }
    }

    println!("WebSocket connection closed.");
}


// Parses the incoming WebSocket message into control values
fn parse_control_data(msg: &str) -> Option<(f32, f32, f32)> {
    let parts: Vec<&str> = msg.split_whitespace().collect();
    if parts.len() != 3 {
        println!("Invalid message format: {}", msg);
        return None;
    }

    let throttle = parts[0].parse::<f32>().ok()?;
    let brake = parts[1].parse::<f32>().ok()?;
    let steering = parts[2].parse::<f32>().ok()?;

    Some((throttle, brake, steering))
}

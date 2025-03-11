use bevy::prelude::*;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::task;
use tokio_tungstenite::accept_async;
use futures_util::stream::StreamExt;
use futures_util::SinkExt; // Added back
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio::sync::Mutex as AsyncMutex;
use tokio::net::TcpStream;

use crate::control::CarControl;

#[derive(Resource)]
pub struct ExternalControls {
    control: Arc<Mutex<CarControl>>,
    last_update: Arc<Mutex<Instant>>, // Track the last time external input was received
}

pub struct ExternalControlPlugin;

impl Plugin for ExternalControlPlugin {
    fn build(&self, app: &mut App) {
        let control = Arc::new(Mutex::new(CarControl::default()));
        let last_update = Arc::new(Mutex::new(Instant::now())); // Initialize last update time

        let websocket_control = control.clone();
        let websocket_last_update = last_update.clone();

        thread::spawn(move || {
            start_websocket_server(websocket_control, websocket_last_update);
        });

        app.insert_resource(ExternalControls { control, last_update })
           .add_systems(Update, update_from_external_controls);
    }
}

// Receive WebSocket messages and update CarControl
fn update_from_external_controls(
    external_controls: Res<ExternalControls>,
    mut car_control: ResMut<CarControl>,
) {
    if let Ok(last_update) = external_controls.last_update.lock() {
        if last_update.elapsed().as_secs() > 5 {
            static mut LAST_LOG_TIME: Option<Instant> = None;
            unsafe {
                if LAST_LOG_TIME.map_or(true, |t| t.elapsed().as_secs() >= 5) {
                    println!("No recent input, skipping update.");
                    LAST_LOG_TIME = Some(Instant::now());
                }
            }
            return;
        }
    }

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

            send_control_update_to_server(car_control.throttle, car_control.brake, car_control.steering);
        }
    }    
}

// WebSocket Server to listen for external inputs
fn start_websocket_server(control: Arc<Mutex<CarControl>>, last_update: Arc<Mutex<Instant>>) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async move {
        let listener = TcpListener::bind("127.0.0.1:8080").await.expect("Failed to bind WebSocket server");
        println!("Bevy WebSocket server listening on ws://127.0.0.1:8080");

        while let Ok((stream, _)) = listener.accept().await {
            println!("New WebSocket connection established.");
            let ws_stream = accept_async(stream).await.expect("Failed to accept WebSocket connection");
            let control_clone = control.clone();
            let last_update_clone = last_update.clone();

            task::spawn(handle_websocket_connection(ws_stream, control_clone, last_update_clone));
        }
    });
}

// Handle incoming WebSocket messages
async fn handle_websocket_connection(
    mut ws_stream: WebSocketStream<TcpStream>,
    control: Arc<Mutex<CarControl>>,
    last_update: Arc<Mutex<Instant>>,
) {
    println!("Listening for incoming WebSocket messages...");
    
    while let Some(Ok(msg)) = ws_stream.next().await {
        println!("Received WebSocket message: {:?}", msg);
        
        if let Message::Text(text) = msg {
            println!("Parsed text: {}", text);
            
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

static WS_CONNECTION: OnceLock<AsyncMutex<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>> = OnceLock::new();

// Send control updates back to the WebSocket server via a persistent connection
fn send_control_update_to_server(throttle: f32, brake: f32, steering: f32) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async move {
        println!(
            "Sending control update - Throttle: {:.2}, Brake: {:.2}, Steering: {:.2}",
            throttle, brake, steering
        );

        let ws = WS_CONNECTION.get_or_init(|| AsyncMutex::new(None));
        let mut ws_guard = ws.lock().await;

        if ws_guard.is_none() {
            match connect_async("ws://127.0.0.1:8080").await {
                Ok((stream, _)) => {
                    println!("Connected to WebSocket server for control updates.");
                    *ws_guard = Some(stream);
                }
                Err(e) => {
                    eprintln!("Failed to connect to WebSocket server: {}", e);
                    return;
                }
            }
        }

        if let Some(ws_stream) = ws_guard.as_mut() {
            let data = format!("{:.2} {:.2} {:.2}", throttle, brake, steering);
            println!("Sending WebSocket message: {}", data);

            if let Err(e) = ws_stream.send(Message::Text(data)).await {
                eprintln!("Failed to send control update: {}", e);
                *ws_guard = None; // Reset connection if sending fails
            }
        }
    });
}

use std::{env::args, net::SocketAddr, thread};

use enigo::MouseControllable;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use warp::{
    reject::{Reject, Rejection},
    reply::Reply,
    Filter,
};

#[derive(Serialize, Deserialize, Debug)]
enum MouseButton {
    Left,
    Middle,
    Right,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
}

impl From<MouseButton> for enigo::MouseButton {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Left => enigo::MouseButton::Left,
            MouseButton::Middle => enigo::MouseButton::Middle,
            MouseButton::Right => enigo::MouseButton::Right,
            MouseButton::ScrollUp => enigo::MouseButton::ScrollUp,
            MouseButton::ScrollDown => enigo::MouseButton::ScrollDown,
            MouseButton::ScrollLeft => enigo::MouseButton::ScrollLeft,
            MouseButton::ScrollRight => enigo::MouseButton::ScrollRight,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum MouseAction {
    MouseDown(MouseButton),
    MouseUp(MouseButton),
    MouseMove(f32, f32),
}

#[derive(Serialize, Deserialize, Debug)]
struct MouseActionMessage {
    action: MouseAction,
}

#[derive(Serialize, Deserialize, Debug)]
struct MouseActionsMessage {
    actions: Vec<MouseAction>,
    delay_between: Option<u64>,
}

#[tokio::main]
async fn main() {
    let mut ip = [127, 0, 0, 1] as [u8; 4];
    let mut port = 8080 as u16;
    for (i, arg) in args().enumerate() {
        if arg == "--server" {
            let ip_temp = args()
                .nth(i + 1)
                .unwrap()
                .split(".")
                .map(|f| f.parse::<u8>().unwrap())
                .collect::<Vec<u8>>();

            ip = [ip_temp[0], ip_temp[1], ip_temp[2], ip_temp[3]];
        } else if arg == "--port" {
            port = args().nth(i + 1).unwrap().parse::<u16>().unwrap();
        }
    }
    pretty_env_logger::init();

    let _post_mouse_action = warp::post()
        .and(warp::path("mouse-action"))
        .and(warp::body::json())
        .map(post_mouse_action_handler);

    let post_mouse_actions = warp::post()
        .and(warp::path("mouse-actions"))
        .and(warp::body::json())
        .map(post_mouse_actions_handler)
        .map(handle_error);
    let sock_addr: SocketAddr = (ip, port).into();
    println!("Server running at {}", sock_addr);
    warp::serve(post_mouse_actions).run((ip, port)).await;
}

fn post_mouse_actions_handler(actions: MouseActionsMessage) -> Result<Value, Rejection> {
    let mut results = Vec::new();
    print!("{:?}", actions);
    for action in actions.actions {
        let res = post_mouse_action_handler(MouseActionMessage { action });
        results.push(res);
        if let Some(d) = actions.delay_between {
            thread::sleep(std::time::Duration::from_millis(d));
        }
    }
    Ok(json!({"messages": results}))
}

fn post_mouse_action_handler(action: MouseActionMessage) -> Value {
    let mut mouse = enigo::Enigo::new();
    match action.action {
        MouseAction::MouseDown(button) => {
            let message = format!("Mouse down button: {:?}", &button);
            println!("{}", message);
            mouse.mouse_down(button.into());
            json!({"message": message})
        }
        MouseAction::MouseUp(button) => {
            let message = format!("Mouse up button: {:?}", &button);
            println!("{}", message);
            mouse.mouse_up(button.into());
            json!({"message": message})
        }
        MouseAction::MouseMove(x, y) => {
            let message = format!("Mouse move to ({}, {})", x, y);
            println!("{}", message);
            mouse.mouse_move_to(x as i32, y as i32);
            json!({"message": message})
        }
    }
}

fn handle_error(result: Result<Value, Rejection>) -> impl Reply {
    match result {
        Ok(value) => warp::reply::json(&value).into_response(),
        Err(rejection) => {
            if rejection.is_not_found() {
                warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND)
                    .into_response()
            } else if let Some(MouseUpError) = rejection.find() {
                warp::reply::with_status(
                    "Mouse up error",
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                )
                .into_response()
            } else {
                warp::reply::with_status(
                    "Internal server error",
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                )
                .into_response()
            }
        }
    }
}

#[derive(Debug)]
struct MouseMoveError;
impl Reject for MouseMoveError {}

#[derive(Debug)]
struct MouseDownError;
impl Reject for MouseDownError {}

#[derive(Debug)]
struct MouseUpError;
impl Reject for MouseUpError {}

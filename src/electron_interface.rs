use crate::ui_session_interface::InvokeUiSession;
use crate::client::QualityStatus;
use hbb_common::{
    log,
    message_proto::*,
    rendezvous_proto::ConnType,
    tokio::{
        self,
        net::TcpListener,
        sync::mpsc,
    },
    ResultType,
};
use scrap::ImageRgb;
use std::sync::Arc;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use hbb_common::futures::{SinkExt, StreamExt};
use async_trait::async_trait;

use crate::client::Data;
use serde::Deserialize;

use std::sync::RwLock;

#[derive(Clone, Default)]
pub struct ElectronUiHandler {
    video_sender: Arc<std::sync::Mutex<Option<mpsc::UnboundedSender<Vec<u8>>>>>,
    input_sender: Arc<std::sync::RwLock<Option<mpsc::UnboundedSender<Data>>>>,
}

impl ElectronUiHandler {
    pub fn new(video_sender: mpsc::UnboundedSender<Vec<u8>>, input_sender: Arc<std::sync::RwLock<Option<mpsc::UnboundedSender<Data>>>>) -> Self {
        Self {
            video_sender: Arc::new(std::sync::Mutex::new(Some(video_sender))),
            input_sender,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum InputEvent {
    #[serde(rename = "mousemove")]
    MouseMove { x: i32, y: i32 },
    #[serde(rename = "mousedown")]
    MouseDown { btn: String, x: i32, y: i32 },
    #[serde(rename = "mouseup")]
    MouseUp { btn: String, x: i32, y: i32 },
    #[serde(rename = "wheel")]
    Wheel { delta_x: i32, delta_y: i32 },
    #[serde(rename = "keydown")]
    KeyDown { key: String },
    #[serde(rename = "keyup")]
    KeyUp { key: String },
}

impl InvokeUiSession for ElectronUiHandler {
    fn set_cursor_data(&self, cd: CursorData) {
        use hbb_common::sodiumoxide::base64;
        let png_data = base64::encode(&cd.colors, base64::Variant::Original);
        let msg = serde_json::json!({
            "type": "cursor_data",
            "id": cd.id,
            "hotx": cd.hotx,
            "hoty": cd.hoty,
            "width": cd.width,
            "height": cd.height,
            "data": png_data
        });
        if let Some(sender) = self.video_sender.lock().unwrap().as_ref() {
             // Send as text message (bytes)
             let text = msg.to_string();
             // We use the same channel for video (binary) and other messages?
             // The channel is Vec<u8>.
             // In start_electron_server, we handle rx.recv().
             // If it's binary, we send Message::Binary.
             // If we want to send Text, we need to differentiate.
             // But the channel only accepts Vec<u8>.
             // I should probably wrap it or use a convention.
             // Current video frame: width(4) + height(4) + data.
             // If I send JSON, I can prefix it with a magic number or just use a different channel?
             // Or, since video frames are huge and JSON is small...
             // Let's use a magic header for JSON?
             // Or better: The `video_sender` is `mpsc::UnboundedSender<Vec<u8>>`.
             // The receiver in `start_electron_server` sends `Message::Binary`.
             // Electron `renderer.js` expects binary video frames.
             
             // Problem: `renderer.js` `ws.onmessage` assumes everything is a video frame!
             // I need to change `renderer.js` to handle both.
             // I can add a header byte: 0 = Video, 1 = JSON.
             
             let mut data = Vec::new();
             data.push(1); // 1 = JSON
             data.extend_from_slice(text.as_bytes());
             sender.send(data).ok();
        }
    }
    fn set_cursor_id(&self, _id: String) {}
    fn set_cursor_position(&self, cp: CursorPosition) {
        let msg = serde_json::json!({
            "type": "cursor_position",
            "x": cp.x,
            "y": cp.y
        });
        if let Some(sender) = self.video_sender.lock().unwrap().as_ref() {
             let text = msg.to_string();
             let mut data = Vec::new();
             data.push(1); // 1 = JSON
             data.extend_from_slice(text.as_bytes());
             sender.send(data).ok();
        }
    }
    fn set_display(&self, _x: i32, _y: i32, _w: i32, _h: i32, _cursor_embedded: bool, _scale: f64) {}
    fn switch_display(&self, _display: &SwitchDisplay) {}
    fn set_peer_info(&self, _peer_info: &PeerInfo) {}
    fn set_displays(&self, _displays: &Vec<DisplayInfo>) {}
    fn set_platform_additions(&self, _data: &str) {}
    fn on_connected(&self, _conn_type: ConnType) {
        log::info!("ElectronUiHandler: Connected!");
    }
    fn update_privacy_mode(&self) {}
    fn set_permission(&self, _name: &str, _value: bool) {}
    fn close_success(&self) {}
    fn update_quality_status(&self, _qs: QualityStatus) {}
    fn set_connection_type(&self, is_secured: bool, direct: bool, stream_type: &str) {
        println!("Connection Established:");
        println!("  - Secured: {}", is_secured);
        println!("  - Direct (P2P): {}", direct);
        println!("  - Stream Type: '{}'", stream_type);
        if direct {
            println!("  => Mode: Direct Connection");
        } else {
            println!("  => Mode: Relay Connection");
        }
    }
    fn set_fingerprint(&self, _fingerprint: String) {}
    fn job_error(&self, _id: i32, _err: String, _file_num: i32) {}
    fn job_done(&self, _id: i32, _file_num: i32) {}
    fn clear_all_jobs(&self) {}
    fn new_message(&self, _msg: String) {}
    fn update_transfer_list(&self) {}
    fn load_last_job(&self, _cnt: i32, _job_json: &str, _auto_start: bool) {}
    fn update_folder_files(
        &self,
        _id: i32,
        _entries: &Vec<FileEntry>,
        _path: String,
        _is_local: bool,
        _only_count: bool,
    ) {
    }
    fn confirm_delete_files(&self, _id: i32, _i: i32, _name: String) {}
    fn override_file_confirm(
        &self,
        _id: i32,
        _file_num: i32,
        _to: String,
        _is_upload: bool,
        _is_identical: bool,
    ) {
    }
    fn update_block_input_state(&self, _on: bool) {}
    fn job_progress(&self, _id: i32, _file_num: i32, _speed: f64, _finished_size: f64) {}
    fn adapt_size(&self) {}
    
    fn on_rgba(&self, _display: usize, rgba: &mut ImageRgb) {
        // Simple serialization: type (1 byte) + width (4 bytes) + height (4 bytes) + raw data
        // Type 0 = Video Frame
        log::info!("ElectronUiHandler::on_rgba: {}x{}, len: {}", rgba.w, rgba.h, rgba.raw.len());
        
        // Swap BGR to RGB (or vice versa)
        // RustDesk usually uses BGRA internally on Windows. HTML5 Canvas expects RGBA.
        // We need to swap the 0th (B) and 2nd (R) byte of every 4-byte pixel.
        for chunk in rgba.raw.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }

        let mut data = Vec::with_capacity(1 + 8 + rgba.raw.len());
        data.push(0); // 0 = Video Frame
        data.extend_from_slice(&(rgba.w as u32).to_le_bytes());
        data.extend_from_slice(&(rgba.h as u32).to_le_bytes());
        data.extend_from_slice(&rgba.raw);

        if let Some(sender) = self.video_sender.lock().unwrap().as_ref() {
            if let Err(e) = sender.send(data) {
                log::error!("Failed to send video frame to channel: {}", e);
            } else {
                log::info!("Sent video frame to Electron channel");
            }
        } else {
            log::error!("video_sender is None!");
        }
    }

    fn msgbox(&self, _msgtype: &str, _title: &str, _text: &str, _link: &str, _retry: bool) {
        log::info!("msgbox: type={}, title={}, text={}, link={}, retry={}", _msgtype, _title, _text, _link, _retry);
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    fn clipboard(&self, _content: String) {}
    fn cancel_msgbox(&self, _tag: &str) {}
    fn switch_back(&self, _id: &str) {}
    fn portable_service_running(&self, _running: bool) {}
    fn on_voice_call_started(&self) {}
    fn on_voice_call_closed(&self, _reason: &str) {}
    fn on_voice_call_waiting(&self) {}
    fn on_voice_call_incoming(&self) {}
    fn get_rgba(&self, _display: usize) -> *const u8 {
        std::ptr::null()
    }
    fn next_rgba(&self, _display: usize) {}
    #[cfg(all(feature = "vram", feature = "flutter"))]
    fn on_texture(&self, _display: usize, _texture: *mut c_void) {}
    fn set_multiple_windows_session(&self, _sessions: Vec<WindowsSession>) {}
    fn set_current_display(&self, _disp_idx: i32) {}
    #[cfg(feature = "flutter")]
    fn is_multi_ui_session(&self) -> bool { false }
    fn update_record_status(&self, _start: bool) {}
    fn printer_request(&self, _id: i32, _path: String) {}
    fn handle_screenshot_resp(&self, _sid: String, _msg: String) {}
    fn handle_terminal_response(&self, _response: TerminalResponse) {}
}

pub async fn start_electron_server(port: u16, input_sender: Arc<std::sync::RwLock<Option<mpsc::UnboundedSender<Data>>>>) -> ResultType<ElectronUiHandler> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    log::info!("Electron WebSocket server listening on: {}", addr);

    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let handler = ElectronUiHandler::new(tx, input_sender.clone());
    
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            match accept_async(stream).await {
                Ok(mut ws_stream) => {
                    log::info!("New WebSocket connection");
                    
                    loop {
                        tokio::select! {
                            Some(frame_data) = rx.recv() => {
                                log::info!("Sending video frame to WebSocket, size: {}", frame_data.len());
                                let msg = Message::Binary(frame_data.into());
                                if let Err(e) = ws_stream.send(msg).await {
                                    log::error!("Error sending video frame: {}", e);
                                    break;
                                }
                            }
                            msg = ws_stream.next() => {
                                match msg {
                                    Some(Ok(Message::Text(text))) => {
                                        if let Ok(event) = serde_json::from_str::<InputEvent>(&text) {
                                            handle_input_event(&input_sender, event);
                                        } else {
                                            log::warn!("Failed to parse input event: {}", text);
                                        }
                                    }
                                    Some(Ok(Message::Close(_))) => break,
                                    Some(Err(e)) => {
                                        log::error!("WebSocket error: {}", e);
                                        break;
                                    }
                                    None => break,
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Error during the websocket handshake: {}", e);
                }
            }
        }
    });

    Ok(handler)
}

fn handle_input_event(sender: &Arc<std::sync::RwLock<Option<mpsc::UnboundedSender<Data>>>>, event: InputEvent) {
    use crate::client::send_mouse;
    use crate::input::{MOUSE_BUTTON_LEFT, MOUSE_BUTTON_RIGHT, MOUSE_TYPE_DOWN, MOUSE_TYPE_UP, MOUSE_TYPE_MOVE, MOUSE_TYPE_WHEEL};

    // Dummy interface struct to satisfy send_mouse signature
    #[derive(Clone)]
    struct DummyInterface {
        sender: Arc<std::sync::RwLock<Option<mpsc::UnboundedSender<Data>>>>,
    }
    #[async_trait]
    impl crate::client::Interface for DummyInterface {
        fn send(&self, data: Data) {
            if let Some(sender) = self.sender.read().unwrap().as_ref() {
                sender.send(data).ok();
            }
        }
        fn get_lch(&self) -> Arc<std::sync::RwLock<crate::client::LoginConfigHandler>> { unimplemented!() }
        fn set_multiple_windows_session(&self, _sessions: Vec<WindowsSession>) {}
        fn msgbox(&self, _msgtype: &str, _title: &str, _text: &str, _link: &str) {}
        fn handle_login_error(&self, _err: &str) -> bool { false }
        fn handle_peer_info(&self, _pi: PeerInfo) {}
        async fn handle_hash(&self, pass: &str, hash: Hash, peer: &mut hbb_common::Stream) {
            use hbb_common::sha2::{Sha256, Digest};
            use hbb_common::protobuf::Message as _;
            
            log::info!("handle_hash called with pass length: {}", pass.len());
            
            // Hash the password with the server's salt
            let mut hasher = Sha256::new();
            hasher.update(pass.as_bytes());
            hasher.update(&hash.salt);
            let hashed_password = hasher.finalize();
            
            // Construct LoginRequest
            let mut login = LoginRequest::new();
            login.username = String::new(); // Will be set by io_loop
            login.password = hbb_common::bytes::Bytes::from(hashed_password.as_slice().to_vec());
            login.my_id = hbb_common::config::Config::get_id();
            login.my_platform = hbb_common::whoami::platform().to_string();
            login.version = crate::VERSION.to_owned();
            
            let mut os_login = OSLogin::new();
            os_login.username = crate::username();
            login.os_login = Some(os_login).into();
            
            let mut msg_out = hbb_common::message_proto::Message::new();
            msg_out.set_login_request(login);
            
            log::info!("Sending LoginRequest with hashed password");
            peer.send(&msg_out).await.ok();
        }
        async fn handle_login_from_ui(&self, _u: String, _p: String, _p2: String, _r: bool, _peer: &mut hbb_common::Stream) {}
        async fn handle_test_delay(&self, _t: TestDelay, _peer: &mut hbb_common::Stream) {}
    }
    
    let interface = DummyInterface { sender: sender.clone() };

    match event {
        InputEvent::MouseMove { x, y } => {
            send_mouse(0, x, y, false, false, false, false, &interface);
        }
        InputEvent::MouseDown { btn, x, y } => {
            let button = match btn.as_str() {
                "left" => MOUSE_BUTTON_LEFT,
                "right" => MOUSE_BUTTON_RIGHT,
                "middle" => 4, // Hardcoded MOUSE_BUTTON_MIDDLE
                _ => 0,
            };
            let mask = (button << 3) | MOUSE_TYPE_DOWN;
            send_mouse(mask, x, y, false, false, false, false, &interface);
        }
        InputEvent::MouseUp { btn, x, y } => {
            let button = match btn.as_str() {
                "left" => MOUSE_BUTTON_LEFT,
                "right" => MOUSE_BUTTON_RIGHT,
                "middle" => 4, // Hardcoded MOUSE_BUTTON_MIDDLE
                _ => 0,
            };
            let mask = (button << 3) | MOUSE_TYPE_UP;
            send_mouse(mask, x, y, false, false, false, false, &interface);
        }
        InputEvent::Wheel { delta_x, delta_y } => {
            let mask = MOUSE_TYPE_WHEEL;
            send_mouse(mask, delta_x, delta_y, false, false, false, false, &interface);
        }
        InputEvent::KeyDown { key } => {
            log::info!("KeyDown: {}", key);
            send_key(&key, true, &interface);
        }
        InputEvent::KeyUp { key } => {
            log::info!("KeyUp: {}", key);
            send_key(&key, false, &interface);
        }
    }
}

fn send_key(key: &str, down: bool, interface: &impl crate::client::Interface) {
    use hbb_common::message_proto::{KeyEvent, ControlKey, KeyboardMode};

    let mut key_event = KeyEvent::new();
    key_event.press = down;
    key_event.mode = KeyboardMode::Legacy.into();

    let lower = key.to_lowercase();
    match lower.as_str() {
        "control" => key_event.set_control_key(ControlKey::Control),
        "shift" => key_event.set_control_key(ControlKey::Shift),
        "alt" => key_event.set_control_key(ControlKey::Alt),
        "meta" => key_event.set_control_key(ControlKey::Meta),
        "enter" => key_event.set_control_key(ControlKey::Return),
        "backspace" => key_event.set_control_key(ControlKey::Backspace),
        "tab" => key_event.set_control_key(ControlKey::Tab),
        "escape" => key_event.set_control_key(ControlKey::Escape),
        "arrowup" => key_event.set_control_key(ControlKey::UpArrow),
        "arrowdown" => key_event.set_control_key(ControlKey::DownArrow),
        "arrowleft" => key_event.set_control_key(ControlKey::LeftArrow),
        "arrowright" => key_event.set_control_key(ControlKey::RightArrow),
        "home" => key_event.set_control_key(ControlKey::Home),
        "end" => key_event.set_control_key(ControlKey::End),
        "pageup" => key_event.set_control_key(ControlKey::PageUp),
        "pagedown" => key_event.set_control_key(ControlKey::PageDown),
        "insert" => key_event.set_control_key(ControlKey::Insert),
        "delete" => key_event.set_control_key(ControlKey::Delete),
        "capslock" => key_event.set_control_key(ControlKey::CapsLock),
        _ => {
            if key.len() == 1 {
                if let Some(c) = key.chars().next() {
                    key_event.set_chr(c as u32);
                }
            } else if key.starts_with("F") && key.len() > 1 {
                if let Ok(n) = key[1..].parse::<i32>() {
                    if n >= 1 && n <= 12 {
                         // Map F1-F12 to ControlKey::F1-F12
                         // Enum values are usually sequential, but better to match explicitly or use from_i32 if available.
                         // For now, let's try to match common F-keys.
                         match n {
                             1 => key_event.set_control_key(ControlKey::F1),
                             2 => key_event.set_control_key(ControlKey::F2),
                             3 => key_event.set_control_key(ControlKey::F3),
                             4 => key_event.set_control_key(ControlKey::F4),
                             5 => key_event.set_control_key(ControlKey::F5),
                             6 => key_event.set_control_key(ControlKey::F6),
                             7 => key_event.set_control_key(ControlKey::F7),
                             8 => key_event.set_control_key(ControlKey::F8),
                             9 => key_event.set_control_key(ControlKey::F9),
                             10 => key_event.set_control_key(ControlKey::F10),
                             11 => key_event.set_control_key(ControlKey::F11),
                             12 => key_event.set_control_key(ControlKey::F12),
                             _ => {}
                         }
                    }
                }
            }
        }
    }

    // Only send if we set something
    if key_event.has_control_key() || key_event.chr() != 0 {
        let mut msg_out = hbb_common::message_proto::Message::new();
        msg_out.set_key_event(key_event);
        interface.send(Data::Message(msg_out));
    } else {
        log::warn!("Unknown key: {}", key);
    }
}

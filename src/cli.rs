use crate::client::*;
use async_trait::async_trait;
use hbb_common::{
    config::PeerConfig,
    config::READ_TIMEOUT,
    futures::{SinkExt, StreamExt},
    log,
    message_proto::*,
    protobuf::Message as _,
    rendezvous_proto::ConnType,
    tokio::{self, sync::mpsc},
    Stream,
};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Session {
    id: String,
    lc: Arc<RwLock<LoginConfigHandler>>,
    sender: mpsc::UnboundedSender<Data>,
    password: String,
}

impl Session {
    pub fn new(id: &str, sender: mpsc::UnboundedSender<Data>, key: &str) -> Self {
        let mut password = "".to_owned();
        if !key.is_empty() {
            password = key.to_owned();
        } else if PeerConfig::load(id).password.is_empty() {
            password = rpassword::prompt_password("Enter password: ").unwrap();
        }
        let session = Self {
            id: id.to_owned(),
            sender,
            password,
            lc: Default::default(),
        };
        session.lc.write().unwrap().initialize(
            id.to_owned(),
            ConnType::DEFAULT_CONN,
            None,
            false,
            None,
            None,
            None,
        );
        session
    }
}

#[async_trait]
impl Interface for Session {
    fn get_lch(&self) -> Arc<RwLock<LoginConfigHandler>> {
        return self.lc.clone();
    }

    fn set_multiple_windows_session(&self, _sessions: Vec<WindowsSession>) {}

    fn msgbox(&self, msgtype: &str, title: &str, text: &str, link: &str) {
        match msgtype {
            "input-password" => {
                self.sender
                    .send(Data::Login(("".to_owned(), "".to_owned(), self.password.clone(), true)))
                    .ok();
            }
            "re-input-password" => {
                log::error!("{}: {}", title, text);
                match rpassword::prompt_password("Enter password: ") {
                    Ok(password) => {
                        let login_data = Data::Login(("".to_owned(), "".to_owned(), password, true));
                        self.sender.send(login_data).ok();
                    }
                    Err(e) => {
                        log::error!("reinput password failed, {:?}", e);
                    }
                }
            }
            msg if msg.contains("error") => {
                log::error!("{}: {}: {}", msgtype, title, text);
            }
            _ => {
                log::info!("{}: {}: {}", msgtype, title, text);
            }
        }
    }

    fn handle_login_error(&self, err: &str) -> bool {
        handle_login_error(self.lc.clone(), err, self)
    }

    fn handle_peer_info(&self, pi: PeerInfo) {
        self.lc.write().unwrap().handle_peer_info(&pi);
    }

    async fn handle_hash(&self, pass: &str, hash: Hash, peer: &mut Stream) {
        log::info!(
            "password={}",
            hbb_common::password_security::temporary_password()
        );
        handle_hash(self.lc.clone(), &pass, hash, self, peer).await;
    }

    async fn handle_login_from_ui(
        &self,
        os_username: String,
        os_password: String,
        password: String,
        remember: bool,
        peer: &mut Stream,
    ) {
        handle_login_from_ui(
            self.lc.clone(),
            os_username,
            os_password,
            password,
            remember,
            peer,
        )
        .await;
    }

    async fn handle_test_delay(&self, t: TestDelay, peer: &mut Stream) {
        handle_test_delay(t, peer).await;
    }

    fn send(&self, data: Data) {
        self.sender.send(data).ok();
    }
}

#[tokio::main(flavor = "current_thread")]
pub async fn connect_test(id: &str, key: String, token: String) {
    let port = 21118;
    // Create a shared sender that will be populated by io_loop
    let sender: Arc<RwLock<Option<mpsc::UnboundedSender<Data>>>> = Default::default();
    
    match crate::electron_interface::start_electron_server(port, sender.clone()).await {
        Ok(handler) => {
            log::info!("Electron server started on port {}", port);
            
            // Auto-launch Electron client
            if let Ok(mut exe_path) = std::env::current_exe() {
                exe_path.pop(); // Get directory
                log::info!("Current executable directory: {:?}", exe_path);

                let client_names = ["sdf-client.exe", "sdfdesk-client.exe"];
                let mut launched = false;

                for name in client_names.iter() {
                    let client_exe = exe_path.join(name);
                    if client_exe.exists() {
                        log::info!("Found client executable: {:?}", client_exe);
                        match std::process::Command::new(&client_exe).spawn() {
                            Ok(_) => {
                                log::info!("Successfully launched client: {:?}", client_exe);
                                launched = true;
                                break;
                            }
                            Err(e) => {
                                log::error!("Failed to launch client {:?}: {}", client_exe, e);
                            }
                        }
                    }
                }

                if !launched {
                    log::error!("Could not find or launch any client executable. Checked: {:?}", client_names);
                    log::info!("Please manually run 'sdf-client.exe' or 'sdfdesk-client.exe'");
                }
            }
            
            let mut session = crate::ui_session_interface::Session {
                password: "".to_owned(),
                args: vec![],
                lc: Default::default(),
                sender: sender, // Use the shared sender
                thread: Default::default(),
                ui_handler: handler,
                server_keyboard_enabled: Default::default(),
                server_file_transfer_enabled: Default::default(),
                server_clipboard_enabled: Default::default(),
                last_change_display: Arc::new(std::sync::Mutex::new(crate::ui_session_interface::ChangeDisplayRecord {
                    time: tokio::time::Instant::now(),
                    display: 0,
                    width: 0,
                    height: 0,
                })),
                connection_round_state: Arc::new(std::sync::Mutex::new(crate::ui_session_interface::ConnectionRoundState {
                    round: 0,
                    state: crate::ui_session_interface::ConnectionState::Disconnected,
                })),
                printer_names: Default::default(),
                reconnect_count: Default::default(),
                last_audit_note: Default::default(),
                audit_guid: Default::default(),
            };

            if !key.is_empty() {
                session.password = key.to_owned();
            } else if PeerConfig::load(id).password.is_empty() {
                session.password = rpassword::prompt_password("Enter password: ").unwrap();
            }

            session.lc.write().unwrap().initialize(
                id.to_owned(),
                ConnType::DEFAULT_CONN,
                None,
                false,
                None,
                None,
                None,
            );

            tokio::task::spawn_blocking(move || {
                log::info!("Starting io_loop in blocking task");
                crate::ui_session_interface::io_loop(session, 0);
            }).await.unwrap();
        }
        Err(e) => log::error!("Failed to start Electron server: {}", e),
    }
}

#[tokio::main(flavor = "current_thread")]
pub async fn start_one_port_forward(
    id: String,
    port: i32,
    remote_host: String,
    remote_port: i32,
    key: String,
    token: String,
) {
    crate::common::test_rendezvous_server();
    crate::common::test_nat_type();
    let (sender, mut receiver) = mpsc::unbounded_channel::<Data>();
    let handler = Session::new(&id, sender, &key);
    if let Err(err) = crate::port_forward::listen(
        handler.id.clone(),
        handler.password.clone(),
        port,
        handler.clone(),
        receiver,
        &key,
        &token,
        handler.lc.clone(),
        remote_host,
        remote_port,
    )
    .await
    {
        log::error!("Failed to listen on {}: {}", port, err);
    }
    log::info!("port forward (:{}) exit", port);
}

#[derive(Clone)]
pub struct CliConnectionManager {}

impl crate::ui_cm_interface::InvokeUiCM for CliConnectionManager {
    fn add_connection(&self, _client: &crate::ui_cm_interface::Client) {}
    fn remove_connection(&self, _id: i32, _close: bool) {}
    fn new_message(&self, _id: i32, _text: String) {}
    fn change_theme(&self, _dark: String) {}
    fn change_language(&self) {}
    fn show_elevation(&self, _show: bool) {}
    fn update_voice_call_state(&self, _client: &crate::ui_cm_interface::Client) {}
    fn file_transfer_log(&self, _action: &str, _log: &str) {}
}

pub fn start_cm_no_ui() {
    let cm = crate::ui_cm_interface::ConnectionManager {
        ui_handler: CliConnectionManager {},
    };
    crate::ui_cm_interface::start_ipc(cm);
}

#[tokio::main(flavor = "current_thread")]
pub async fn start_local_server(id: &str, key: String, _token: String) {
    let port = 21118;
    // Create a shared sender that will be populated by io_loop
    let sender: Arc<RwLock<Option<mpsc::UnboundedSender<Data>>>> = Default::default();
    
    match crate::electron_interface::start_electron_server(port, sender.clone()).await {
        Ok(handler) => {
            log::info!("Electron server started on port {}", port);
            
            // Auto-launch Electron client
            if let Ok(mut exe_path) = std::env::current_exe() {
                exe_path.pop(); // Get directory
                let client_exe = exe_path.join("sdfdesk-client 1.0.0.exe");
                
                log::info!("Attempting to launch client: {:?}", client_exe);
                if let Err(e) = std::process::Command::new(client_exe).spawn() {
                    log::error!("Failed to auto-launch client: {}", e);
                    log::info!("Please manually run 'sdfdesk-client 1.0.0.exe'");
                }
            }
            
            let mut session = crate::ui_session_interface::Session {
                password: "".to_owned(),
                args: vec![],
                lc: Default::default(),
                sender: sender, // Use the shared sender
                thread: Default::default(),
                ui_handler: handler,
                server_keyboard_enabled: Default::default(),
                server_file_transfer_enabled: Default::default(),
                server_clipboard_enabled: Default::default(),
                last_change_display: Arc::new(std::sync::Mutex::new(crate::ui_session_interface::ChangeDisplayRecord {
                    time: tokio::time::Instant::now(),
                    display: 0,
                    width: 0,
                    height: 0,
                })),
                connection_round_state: Arc::new(std::sync::Mutex::new(crate::ui_session_interface::ConnectionRoundState {
                    round: 0,
                    state: crate::ui_session_interface::ConnectionState::Disconnected,
                })),
                printer_names: Default::default(),
                reconnect_count: Default::default(),
                last_audit_note: Default::default(),
                audit_guid: Default::default(),
            };

            if !key.is_empty() {
                session.password = key.to_owned();
            } else if PeerConfig::load(id).password.is_empty() {
                session.password = rpassword::prompt_password("Enter password: ").unwrap();
            }

            session.lc.write().unwrap().initialize(
                id.to_owned(),
                ConnType::DEFAULT_CONN,
                None,
                false,
                None,
                None,
                None,
            );

            std::thread::spawn(move || {
                crate::ui_session_interface::io_loop(session, 0);
            }).join().unwrap();
        }
        Err(e) => log::error!("Failed to start Electron server: {}", e),
    }
}

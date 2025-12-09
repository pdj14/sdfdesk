// RDP Session Management for Headless Mode
// This module provides functionality to auto-create localhost RDP sessions
// for scenarios when a physical display is not connected.

use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use hbb_common::log;

lazy_static::lazy_static! {
    pub static ref RDP_SESSION: Arc<Mutex<Option<RdpSession>>> = Arc::new(Mutex::new(None));
    pub static ref RDP_CREDENTIALS: Arc<Mutex<Option<RdpCredentials>>> = Arc::new(Mutex::new(None));
}

#[derive(Clone)]
pub struct RdpCredentials {
    pub username: String,
    pub password: String,
}

pub struct RdpSession {
    process: Child,
}

impl RdpSession {
    /// Start a localhost RDP session with auto-login
    /// This will disconnect any existing RDP sessions to avoid conflicts
    pub fn start(credentials: &RdpCredentials) -> std::io::Result<Self> {
        log::info!("Starting localhost RDP session for headless mode...");
        log::info!("RDP username: {}", credentials.username);
        
        // Step 1: Disconnect any existing RDP sessions to this machine
        log::info!("Disconnecting existing RDP sessions to avoid conflicts...");
        Self::disconnect_existing_sessions();
        
        // Step 2: Store credentials for localhost RDP
        log::debug!("Storing RDP credentials with cmdkey...");
        let cmdkey_result = Command::new("cmdkey")
            .args(&[
                "/generic:TERMSRV/127.0.0.1",
                &format!("/user:{}", credentials.username),
                &format!("/pass:{}", credentials.password),
            ])
            .output();
        
        match &cmdkey_result {
            Ok(output) => {
                if output.status.success() {
                    log::info!("RDP credentials stored successfully");
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    log::warn!("cmdkey output: {} {}", stdout, stderr);
                }
            }
            Err(e) => {
                log::warn!("Failed to run cmdkey: {}", e);
            }
        }
        
        // Step 3: Create RDP file with auto-login settings
        let rdp_file_path = std::env::temp_dir().join("sdfdesk_rdp.rdp");
        let rdp_content = format!(
            r#"screen mode id:i:2
use multimon:i:0
desktopwidth:i:1920
desktopheight:i:1080
session bpp:i:32
winposstr:s:0,1,0,0,1920,1080
compression:i:1
keyboardhook:i:2
audiocapturemode:i:0
videoplaybackmode:i:1
connection type:i:7
networkautodetect:i:1
bandwidthautodetect:i:1
displayconnectionbar:i:1
enableworkspacereconnect:i:0
disable wallpaper:i:0
allow font smoothing:i:1
allow desktop composition:i:1
disable full window drag:i:0
disable menu anims:i:0
disable themes:i:0
disable cursor setting:i:0
bitmapcachepersistenable:i:1
full address:s:127.0.0.1
audiomode:i:0
redirectprinters:i:0
redirectcomports:i:0
redirectsmartcards:i:0
redirectclipboard:i:1
redirectposdevices:i:0
autoreconnection enabled:i:1
authentication level:i:2
prompt for credentials:i:0
negotiate security layer:i:1
remoteapplicationmode:i:0
alternate shell:s:
shell working directory:s:
gatewayhostname:s:
gatewayusagemethod:i:4
gatewaycredentialssource:i:4
gatewayprofileusagemethod:i:0
promptcredentialonce:i:0
gatewaybrokeringtype:i:0
use redirection server name:i:0
rdgiskdcproxy:i:0
kdcproxyname:s:
username:s:{}
"#,
            credentials.username
        );
        
        if let Err(e) = std::fs::write(&rdp_file_path, &rdp_content) {
            log::warn!("Failed to create RDP file: {}", e);
        } else {
            log::info!("RDP file created at {:?}", rdp_file_path);
        }
        
        // Step 4: Start mstsc with RDP file
        log::debug!("Starting mstsc.exe with RDP file...");
        let process = Command::new("mstsc")
            .arg(rdp_file_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        
        log::info!("RDP session started (PID: {})", process.id());
        log::info!("If login prompt appears, credentials should auto-fill from cmdkey");
        
        // Give some time for RDP to establish
        std::thread::sleep(std::time::Duration::from_secs(3));
        
        Ok(RdpSession { process })
    }
    
    /// Disconnect existing RDP sessions
    fn disconnect_existing_sessions() {
        // Use qwinsta to list sessions and logoff remote ones
        if let Ok(output) = Command::new("qwinsta").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            log::debug!("Current sessions:\n{}", stdout);
            
            // Parse session IDs and logoff RDP sessions
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let Ok(session_id) = parts[2].parse::<u32>() {
                        if session_id != 0 && (line.contains("rdp-tcp") || line.contains("Disc")) {
                            log::info!("Logging off session {}", session_id);
                            let _ = Command::new("logoff")
                                .arg(session_id.to_string())
                                .status();
                        }
                    }
                }
            }
        }
    }
    
    
    /// Stop the RDP session
    pub fn stop(&mut self) -> std::io::Result<()> {
        log::info!("Stopping RDP session...");
        
        // Kill the mstsc process
        if let Err(e) = self.process.kill() {
            log::warn!("Failed to kill RDP process: {}", e);
        }
        
        // Clean up stored credentials
        let _ = Command::new("cmdkey")
            .args(&["/delete:127.0.0.1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        
        // Also try to logoff the RDP session
        let _ = Command::new("cmd")
            .args(&["/c", "logoff"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        
        log::info!("RDP session stopped");
        Ok(())
    }
    
    /// Check if RDP session is still running
    pub fn is_running(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(Some(_)) => false, // Process exited
            Ok(None) => true,     // Still running
            Err(_) => false,
        }
    }
}

impl Drop for RdpSession {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Set RDP credentials for headless mode
pub fn set_rdp_credentials(username: String, password: String) {
    let credentials = RdpCredentials { username, password };
    *RDP_CREDENTIALS.lock().unwrap() = Some(credentials);
    log::info!("RDP mode enabled for headless operation");
}

/// Check if RDP mode is enabled
pub fn is_rdp_mode_enabled() -> bool {
    RDP_CREDENTIALS.lock().unwrap().is_some()
}

/// Get RDP credentials
pub fn get_rdp_credentials() -> Option<RdpCredentials> {
    RDP_CREDENTIALS.lock().unwrap().clone()
}

/// Start RDP session if in RDP mode and no session exists
pub fn start_rdp_session_if_needed() -> std::io::Result<bool> {
    if !is_rdp_mode_enabled() {
        return Ok(false);
    }
    
    let mut session_guard = RDP_SESSION.lock().unwrap();
    
    // Check if session already exists and is running
    if let Some(ref mut session) = *session_guard {
        if session.is_running() {
            return Ok(true);
        }
    }
    
    // Start new RDP session
    if let Some(credentials) = get_rdp_credentials() {
        let session = RdpSession::start(&credentials)?;
        *session_guard = Some(session);
        
        // Give RDP some time to establish the virtual display
        std::thread::sleep(std::time::Duration::from_secs(3));
        
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Stop RDP session
pub fn stop_rdp_session() {
    let mut session_guard = RDP_SESSION.lock().unwrap();
    if let Some(ref mut session) = *session_guard {
        let _ = session.stop();
    }
    *session_guard = None;
}

/// Check if display is connected (for deciding whether to start RDP)
pub fn has_display_connected() -> bool {
    // Use scrap crate's display enumeration which is already proven to work
    match scrap::Display::all() {
        Ok(displays) => {
            log::info!("Display detection: found {} displays", displays.len());
            for (i, d) in displays.iter().enumerate() {
                log::debug!("Display {}: {}x{}", i, d.width(), d.height());
            }
            !displays.is_empty()
        }
        Err(e) => {
            log::warn!("Failed to enumerate displays: {:?}", e);
            // If we can't enumerate displays, try with primary display
            match scrap::Display::primary() {
                Ok(d) => {
                    log::info!("Primary display found: {}x{}", d.width(), d.height());
                    true
                }
                Err(e2) => {
                    log::warn!("No primary display found: {:?}", e2);
                    false
                }
            }
        }
    }
}

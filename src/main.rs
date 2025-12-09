// Removed windows_subsystem = "windows" to enable console output for CLI
// CLI applications need console subsystem to display output


use librustdesk::*;

#[cfg(any(target_os = "android", target_os = "ios", feature = "flutter"))]
fn main() {
    if !common::global_init() {
        eprintln!("Global initialization failed.");
        return;
    }
    common::test_rendezvous_server();
    common::test_nat_type();
    common::global_clean();
}

#[cfg(not(any(
    target_os = "android",
    target_os = "ios",
    feature = "cli",
    feature = "flutter"
)))]
fn main() {
    #[cfg(all(windows, not(feature = "inline")))]
    unsafe {
        winapi::um::shellscalingapi::SetProcessDpiAwareness(2);
    }
    if let Some(args) = crate::core_main::core_main().as_mut() {
        ui::start(args);
    }
    common::global_clean();
}

#[cfg(feature = "cli")]
fn main() {
    if !common::global_init() {
        return;
    }
    // Set APP_NAME to "sdfdesk" to enable portable config logic
    *hbb_common::config::APP_NAME.write().unwrap() = "sdfdesk".to_owned();
    use clap::{Arg, Command};
    use hbb_common::log;

    let matches = Command::new("sdfdesk")
        .version(crate::VERSION)
        .author("sdfdesk <dj14.park@gmail.com>")
        .about("sdfdesk command line tool")
        .arg(
            Arg::new("port-forward")
                .short('p')
                .long("port-forward")
                .help("Format: remote-id:local-port:remote-port[:remote-host]")
                .num_args(1),
        )
        .arg(
            Arg::new("connect")
                .short('c')
                .long("connect")
                .help("test only")
                .num_args(1),
        )
        .arg(
            Arg::new("unlock-pw")
                .long("unlock-pw")
                .help("OS password for remote unlock")
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("unlock-id")
                .long("unlock-id")
                .help("OS username for remote unlock")
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("key")
                .short('k')
                .long("key")
                .help("")
                .num_args(1),
        )
        .arg(
            Arg::new("server")
                .short('s')
                .long("server")
                .help("Start server")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("direct-server")
                .long("direct-server")
                .help("Enable direct IP access (port 21118)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("password")
                .long("password")
                .help("Set permanent password")
                .num_args(1),
        )
        .arg(
            Arg::new("cm")
                .long("cm")
                .help("Start connection manager")
                .action(clap::ArgAction::SetTrue)
                .hide(true),
        )
        .arg(
            Arg::new("cm-no-ui")
                .long("cm-no-ui")
                .help("Start connection manager without UI")
                .action(clap::ArgAction::SetTrue)
                .hide(true),
        )
        .arg(
            Arg::new("get-id")
                .long("get-id")
                .action(clap::ArgAction::SetTrue)
                .help("Get ID"),
        )
        .arg(
            Arg::new("service")
                .long("service")
                .action(clap::ArgAction::SetTrue)
                .help("Run as service"),
        )
        .arg(
            Arg::new("option")
                .long("option")
                .action(clap::ArgAction::Set)
                .help("Set option"),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .action(clap::ArgAction::Set)
                .help("Set config"),
        )
        .arg(
            Arg::new("install-service")
                .long("install-service")
                .action(clap::ArgAction::SetTrue)
                .help("Install service"),
        )
        .arg(
            Arg::new("uninstall-service")
                .long("uninstall-service")
                .action(clap::ArgAction::SetTrue)
                .help("Uninstall service"),
        )
        .arg(
            Arg::new("start-service")
                .long("start-service")
                .action(clap::ArgAction::SetTrue)
                .help("Start service"),
        )
        .arg(
            Arg::new("stop-service")
                .long("stop-service")
                .action(clap::ArgAction::SetTrue)
                .help("Stop service"),
        )
        .arg(
            Arg::new("set-hbbs")
                .long("set-hbbs")
                .help("Set custom hbbs server (rendezvous server). Use empty string to reset to default")
                .num_args(0..=1)
                .default_missing_value(""),
        )
        .arg(
            Arg::new("set-hbbr")
                .long("set-hbbr")
                .help("Set custom hbbr server (relay server). Use empty string to reset to default")
                .num_args(0..=1)
                .default_missing_value(""),
        )
        .arg(
            Arg::new("set-key")
                .long("set-key")
                .help("Set custom server public key. Use empty string to reset to default")
                .num_args(0..=1)
                .default_missing_value(""),
        )
        .arg(
            Arg::new("show-config")
                .long("show-config")
                .help("Show current server configuration")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("local-server")
                .long("local-server")
                .help("Start local server for Electron")
                .num_args(1),
        )
        .arg(
            Arg::new("rdp-id")
                .long("rdp-id")
                .help("RDP username for headless mode (auto-creates localhost RDP session)")
                .num_args(1),
        )
        .arg(
            Arg::new("rdp-pw")
                .long("rdp-pw")
                .help("RDP password for headless mode")
                .num_args(1),
        )
        .get_matches();

    use hbb_common::config::LocalConfig;
    let _logger_handle = hbb_common::init_log(false, "sdfdesk");

    if let Some(p) = matches.get_one::<String>("port-forward") {
        let options: Vec<String> = p.split(":").map(|x| x.to_owned()).collect();
        if options.len() < 3 {
            log::error!("Wrong port-forward options");
            return;
        }
        let mut port = 0;
        if let Ok(v) = options[1].parse::<i32>() {
            port = v;
        } else {
            log::error!("Wrong local-port");
            return;
        }
        let mut remote_port = 0;
        if let Ok(v) = options[2].parse::<i32>() {
            remote_port = v;
        } else {
            log::error!("Wrong remote-port");
            return;
        }
        let mut remote_host = "localhost".to_owned();
        if options.len() > 3 {
            remote_host = options[3].clone();
        }
        common::test_rendezvous_server();
        common::test_nat_type();
        let key = matches.get_one::<String>("key").map(|s| s.as_str()).unwrap_or("").to_owned();
        let token = LocalConfig::get_option("access_token");
        cli::start_one_port_forward(
            options[0].clone(),
            port,
            remote_host,
            remote_port,
            key,
            token,
        );
    } else if let Some(p) = matches.get_one::<String>("connect") {
        common::test_rendezvous_server();
        common::test_nat_type();
        let key = matches.get_one::<String>("key").map(|s| s.as_str()).unwrap_or("").to_owned();
        let token = LocalConfig::get_option("access_token");
        let unlock_id = matches.get_one::<String>("unlock-id").map(|s| s.to_owned()).unwrap_or_default();
        let unlock_pw = matches.get_one::<String>("unlock-pw").map(|s| s.to_owned()).unwrap_or_default();
        // RDP credentials for headless mode
        let rdp_id = matches.get_one::<String>("rdp-id").map(|s| s.to_owned()).unwrap_or_default();
        let rdp_pw = matches.get_one::<String>("rdp-pw").map(|s| s.to_owned()).unwrap_or_default();
        cli::connect_test(p, key, token, unlock_id, unlock_pw, rdp_id, rdp_pw);
    } else if matches.get_flag("server") {
        let id = hbb_common::config::Config::get_id();
        println!("========================================");
        println!("Server ID: {}", id);
        println!("========================================");
        log::info!("id={}", id);
        log::info!("id={}", id);
        if matches.get_flag("direct-server") {
            hbb_common::config::Config::set_option("direct-server".to_owned(), "Y".to_owned());
            println!("Direct server enabled (port 21118)");
        }
        
        // Check for RDP mode (headless support)
        #[cfg(windows)]
        {
            let rdp_id = matches.get_one::<String>("rdp-id");
            let rdp_pw = matches.get_one::<String>("rdp-pw");
            
            if let (Some(rdp_id), Some(rdp_pw)) = (rdp_id, rdp_pw) {
                crate::rdp_session::set_rdp_credentials(rdp_id.clone(), rdp_pw.clone());
                println!("RDP mode enabled for headless operation");
                
                // Check if display is connected
                if !crate::rdp_session::has_display_connected() {
                    println!("No display detected - will auto-create RDP session on client connect");
                } else {
                    println!("Display detected - RDP mode on standby");
                }
            }
        }
        
        crate::start_server(true, false);
    } else if matches.get_flag("cm") || matches.get_flag("cm-no-ui") {
        crate::cli::start_cm_no_ui();
    } else if let Some(pwd) = matches.get_one::<String>("password") {
        use hbb_common::config::Config;
        Config::set_permanent_password(pwd);
        // Set approve mode to password-only (no UI accept needed)
        Config::set_option("approve-mode".to_owned(), "password".to_owned());
        // Use permanent password only (disable temporary password)
        Config::set_option("verification-method".to_owned(), "use-permanent-password".to_owned());
        println!("Password set successfully");
        println!("Approve mode set to: password-only (no UI accept required)");
    } else if matches.get_flag("get-id") {
        let id = hbb_common::config::Config::get_id();
        println!("{}", id);
    } else if let Some(server) = matches.get_one::<String>("set-hbbs") {
        use hbb_common::config::Config;
        Config::set_option("custom-rendezvous-server".to_owned(), server.to_owned());
        if server.is_empty() {
            println!("HBBS server reset to default (rs-ny.rustdesk.com:21116)");
        } else {
            println!("HBBS server set to: {}", server);
        }
    } else if let Some(server) = matches.get_one::<String>("set-hbbr") {
        use hbb_common::config::Config;
        Config::set_option("relay-server".to_owned(), server.to_owned());
        if server.is_empty() {
            println!("HBBR server reset to default (rs-ny.rustdesk.com:21117)");
        } else {
            println!("HBBR server set to: {}", server);
        }
    } else if let Some(key) = matches.get_one::<String>("set-key") {
        use hbb_common::config::Config;
        Config::set_option("key".to_owned(), key.to_owned());
        if key.is_empty() {
            println!("Server key reset to default");
        } else {
            println!("Server key set successfully");
        }
    } else if matches.get_flag("show-config") {
        use hbb_common::config::Config;
        println!("========================================");
        println!("Current Server Configuration:");
        println!("========================================");
        println!("ID: {}", Config::get_id());
        
        let rendezvous_servers = Config::get_rendezvous_servers();
        if rendezvous_servers.is_empty() {
            println!("HBBS: (default) rs-ny.rustdesk.com");
        } else {
            println!("HBBS: {}", rendezvous_servers.join(", "));
        }
        
        let relay_server = Config::get_option("relay-server");
        if relay_server.is_empty() {
            println!("HBBR: (default) rs-ny.rustdesk.com");
        } else {
            println!("HBBR: {}", relay_server);
        }
        
        // Display configured server key (for connecting to custom servers)
        let server_key = Config::get_option("key");
        if server_key.is_empty() {
            println!("Key: (not set)");
        } else {
            println!("Key: {}", server_key);
        }
        
        println!("========================================");
    } else if let Some(id) = matches.get_one::<String>("local-server") {
        common::test_rendezvous_server();
        common::test_nat_type();
        let key = matches.get_one::<String>("key").map(|s| s.as_str()).unwrap_or("").to_owned();
        let token = LocalConfig::get_option("access_token");
        cli::start_local_server(id, key, token);
    } else if matches.get_flag("service") {
        if matches.get_flag("direct-server") {
            hbb_common::config::Config::set_option("direct-server".to_owned(), "Y".to_owned());
        }
        #[cfg(target_os = "windows")]
        {
            // Debug: Log config path and password status for troubleshooting
            use hbb_common::config::Config;
            let config_file = Config::file();
            log::info!("[Service] Config file path: {:?}", config_file);
            log::info!("[Service] current_exe: {:?}", std::env::current_exe());
            let permanent_password = Config::get_permanent_password();
            log::info!("[Service] Permanent password length: {}", permanent_password.len());
            log::info!("[Service] Permanent password is empty: {}", permanent_password.is_empty());
            
            librustdesk::start_os_service();
        }
    } else if matches.get_flag("install-service") {
        #[cfg(target_os = "windows")]
        {
            let exe = std::env::current_exe().unwrap();
            let exe_path = exe.to_str().unwrap();
            let app_name = "sdfdesk";
            let display_name = "sdfdesk Service";
            
            // Use direct execution to avoid cmd quoting hell
            // binpath= "\"C:\Path\To\exe\" --service"
            let mut binpath = format!("\"{}\" --service", exe_path);
            if matches.get_flag("direct-server") {
                binpath.push_str(" --direct-server");
            }
            
            println!("Installing service...");
            let status = std::process::Command::new("sc")
                .arg("create")
                .arg(app_name)
                .arg("binpath=")
                .arg(&binpath)
                .arg("start=")
                .arg("demand")
                .arg("DisplayName=")
                .arg(display_name)
                .status();
                
            match status {
                Ok(s) => {
                    if s.success() {
                        println!("Service installed successfully.");
                        println!("Run 'sdfdesk --start-service' to start it.");
                    } else {
                        eprintln!("Failed to install service. Exit code: {:?}", s.code());
                    }
                }
                Err(e) => eprintln!("Failed to execute sc create: {}", e),
            }
        }
    } else if matches.get_flag("start-service") {
        #[cfg(target_os = "windows")]
        {
            let app_name = "sdfdesk";
            let exe = std::env::current_exe().unwrap();
            let exe_path = exe.to_str().unwrap();
            let display_name = "sdfdesk Service";
            let mut binpath = format!("\"{}\" --service", exe_path);
            if matches.get_flag("direct-server") {
                binpath.push_str(" --direct-server");
            }

            // 1. Check if service exists
            let query = std::process::Command::new("sc")
                .arg("query")
                .arg(app_name)
                .output();
            
            let exists = match query {
                Ok(output) => output.status.success(),
                Err(_) => false,
            };

            if !exists {
                println!("Service not found. Installing...");
                let create_res = std::process::Command::new("sc")
                    .arg("create")
                    .arg(app_name)
                    .arg("binpath=")
                    .arg(&binpath)
                    .arg("start=")
                    .arg("demand")
                    .arg("DisplayName=")
                    .arg(display_name)
                    .output();

                match create_res {
                    Ok(output) => {
                        if !output.status.success() {
                            println!("Failed to create service: {}", String::from_utf8_lossy(&output.stdout));
                            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
                        } else {
                            println!("Service created successfully.");
                        }
                    }
                    Err(e) => eprintln!("Failed to execute sc create: {}", e),
                }
            } else {
                println!("Service exists. Updating configuration...");
                // Update binpath in case exe moved or changed
                let _ = std::process::Command::new("sc")
                    .arg("config")
                    .arg(app_name)
                    .arg("binpath=")
                    .arg(&binpath)
                    .arg("start=")
                    .arg("demand")
                    .output();
            }

            // 2. Start service
            println!("Starting service...");
            let start = std::process::Command::new("sc")
                .arg("start")
                .arg(app_name)
                .output();
                
            match start {
                Ok(output) => {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    if !output.stderr.is_empty() {
                        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
                    }
                }
                Err(e) => eprintln!("Failed to start service: {}", e),
            }
        }
    } else if matches.get_flag("stop-service") {
        #[cfg(target_os = "windows")]
        {
            let app_name = "sdfdesk";
            println!("Stopping service...");
            let stop = std::process::Command::new("sc")
                .arg("stop")
                .arg(app_name)
                .output();
                
            match stop {
                Ok(output) => {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    if !output.stderr.is_empty() {
                        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
                    }
                }
                Err(e) => eprintln!("Failed to stop service: {}", e),
            }
        }
    } else if matches.get_flag("uninstall-service") {
        #[cfg(target_os = "windows")]
        {
            let app_name = "sdfdesk";
            println!("Stopping service...");
            let _ = std::process::Command::new("sc")
                .arg("stop")
                .arg(app_name)
                .output();
                
            println!("Deleting service...");
            let delete = std::process::Command::new("sc")
                .arg("delete")
                .arg(app_name)
                .output();
                
            match delete {
                Ok(output) => {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    if !output.stderr.is_empty() {
                        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
                    }
                }
                Err(e) => eprintln!("Failed to delete service: {}", e),
            }
        }
    }
    common::global_clean();
}

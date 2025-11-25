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
                .help("Get server ID")
                .action(clap::ArgAction::SetTrue),
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
        .get_matches();

    use hbb_common::{config::LocalConfig, env_logger::*};
    init_from_env(Env::default().filter_or(DEFAULT_FILTER_ENV, "info"));

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
        cli::connect_test(p, key, token);
    } else if matches.get_flag("server") {
        let id = hbb_common::config::Config::get_id();
        println!("========================================");
        println!("Server ID: {}", id);
        println!("========================================");
        log::info!("id={}", id);
        crate::start_server(true, false);
    } else if matches.get_flag("cm") || matches.get_flag("cm-no-ui") {
        crate::cli::start_cm_no_ui();
    } else if let Some(pwd) = matches.get_one::<String>("password") {
        use hbb_common::config::Config;
        Config::set_permanent_password(pwd);
        println!("Password set successfully");
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
    }
    common::global_clean();
}

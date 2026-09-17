use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let args = match cli::Args::parse() {
        Ok(args) => args,
        Err(message) => {
            eprintln!("error: {message}\n\n{}", cli::usage());
            return ExitCode::from(2);
        }
    };

    if args.help {
        println!("{}", cli::usage());
        return ExitCode::SUCCESS;
    }
    if args.version {
        println!("portopsy {VERSION}");
        return ExitCode::SUCCESS;
    }

    match platform::run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(1)
        }
    }
}

mod cli {
    use std::env;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ProtocolFilter {
        Tcp,
        Udp,
        All,
    }

    #[derive(Debug)]
    pub struct Args {
        pub port: Option<u16>,
        pub protocol: ProtocolFilter,
        pub json: bool,
        pub resolve: bool,
        pub no_color: bool,
        pub help: bool,
        pub version: bool,
    }

    impl Args {
        pub fn parse() -> Result<Self, String> {
            Self::parse_from(env::args().skip(1))
        }

        fn parse_from<I, S>(arguments: I) -> Result<Self, String>
        where
            I: IntoIterator<Item = S>,
            S: Into<String>,
        {
            let mut port = None;
            let mut protocol = ProtocolFilter::All;
            let mut json = false;
            let mut resolve = false;
            let mut no_color = false;
            let mut help = false;
            let mut version = false;

            let mut arguments = arguments.into_iter();
            while let Some(raw_argument) = arguments.next() {
                let argument: String = raw_argument.into();
                match argument.as_str() {
                    "-h" | "--help" => help = true,
                    "-V" | "--version" => version = true,
                    "--json" => json = true,
                    "--resolve" | "-r" => resolve = true,
                    "--no-color" | "-n" => no_color = true,
                    "-p" | "--port" => {
                        let value = arguments
                            .next()
                            .ok_or_else(|| "--port requires a value".to_string())?
                            .into();
                        if port.is_some() {
                            return Err("only one port may be supplied".to_string());
                        }
                        port = Some(parse_port(&value)?);
                    }
                    "--tcp" => protocol = ProtocolFilter::Tcp,
                    "--udp" => protocol = ProtocolFilter::Udp,
                    "--all" => protocol = ProtocolFilter::All,
                    value if value.strip_prefix("--port=").is_some() => {
                        let value = value.strip_prefix("--port=").unwrap();
                        if port.is_some() {
                            return Err("only one port may be supplied".to_string());
                        }
                        port = Some(parse_port(value)?);
                    }
                    value if value.starts_with("-p") && value.len() > 2 => {
                        let value = &value[2..];
                        if port.is_some() {
                            return Err("only one port may be supplied".to_string());
                        }
                        port = Some(parse_port(value)?);
                    }
                    value if value.starts_with('-') => {
                        return Err(format!("unknown option: {value}"));
                    }
                    value => {
                        if port.is_some() {
                            return Err("only one port may be supplied".to_string());
                        }
                        port = Some(parse_port(value)?);
                    }
                }
            }

            if !help && !version && port.is_none() {
                return Err("a port is required".to_string());
            }
            if json && resolve {
                return Err("--json and --resolve cannot be used together".to_string());
            }

            Ok(Self {
                port,
                protocol,
                json,
                resolve,
                no_color,
                help,
                version,
            })
        }
    }

    pub fn parse_port(value: &str) -> Result<u16, String> {
        let value = value
            .strip_prefix("tcp://")
            .or_else(|| value.strip_prefix("udp://"))
            .unwrap_or(value)
            .trim_start_matches(':');
        let value = value.rsplit(':').next().unwrap_or(value);
        let port = value
            .parse::<u16>()
            .map_err(|_| format!("invalid port: {value}"))?;
        if port == 0 {
            return Err("port must be between 1 and 65535".to_string());
        }
        Ok(port)
    }

    pub fn usage() -> &'static str {
        "portopsy: explain what is occupying a TCP or UDP port\n\nUSAGE:\n    portopsy [OPTIONS] <PORT>\n\nEXAMPLES:\n    portopsy :3000\n    portopsy --tcp 8080\n    portopsy --udp 5353\n    portopsy 0.0.0.0:3000\n    portopsy --port 3000\n    portopsy --json :3000\n\nOPTIONS:\n    --tcp       inspect TCP sockets only\n    --udp       inspect UDP sockets only\n    --all       inspect TCP and UDP sockets (default)\n    --port, -p  accept 3000, :3000, or host:port syntax\n    --json      emit machine-readable JSON\n    --resolve, -r  interactively choose and execute one safe resolution action\n    -h, --help  show this help\n    -V, --version  show version\n\nColors are enabled in interactive terminals. Use --no-color or NO_COLOR=1 to disable them.\n\n--resolve always asks you to choose a target, choose an action, and type 'yes' before execution."
    }

    #[cfg(test)]
    mod tests {
        use super::{parse_port, ProtocolFilter};

        #[test]
        fn parses_common_port_forms() {
            assert_eq!(parse_port("3000").unwrap(), 3000);
            assert_eq!(parse_port(":3000").unwrap(), 3000);
            assert_eq!(parse_port("tcp://:8080").unwrap(), 8080);
            assert_eq!(parse_port("udp://127.0.0.1:5353").unwrap(), 5353);
        }

        #[test]
        fn rejects_invalid_ports() {
            assert!(parse_port("0").is_err());
            assert!(parse_port("65536").is_err());
            assert!(parse_port("abc").is_err());
        }

        #[test]
        fn protocol_filter_is_copyable() {
            let filter = ProtocolFilter::Tcp;
            assert_eq!(filter, ProtocolFilter::Tcp);
        }
        #[test]
        fn resolve_requires_explicit_non_json_mode() {
            let args = super::Args::parse_from(["--resolve", ":3000"])
                .expect("resolve arguments should parse");
            assert!(args.resolve);
            assert!(super::Args::parse_from(["--json", "--resolve", ":3000"]).is_err());
            assert!(super::Args::parse_from(["-r", ":3000"]).unwrap().resolve);
        }
        #[test]
        fn parses_aliases_and_host_port_forms() {
            assert_eq!(super::Args::parse_from(["3000"]).unwrap().port, Some(3000));
            assert_eq!(super::Args::parse_from([":3000"]).unwrap().port, Some(3000));
            assert_eq!(
                super::Args::parse_from(["0.0.0.0:3000"]).unwrap().port,
                Some(3000)
            );
            assert_eq!(
                super::Args::parse_from(["-p", "3000"]).unwrap().port,
                Some(3000)
            );
            assert_eq!(
                super::Args::parse_from(["--port=3000"]).unwrap().port,
                Some(3000)
            );
            assert!(super::Args::parse_from(["--port"]).is_err());
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use crate::cli::{Args, ProtocolFilter};
    use std::collections::{HashMap, HashSet};
    use std::env;
    use std::fs;
    use std::io::{self, IsTerminal, Write};
    use std::path::{Path, PathBuf};
    use std::process::Command;
    const SIGTERM: i32 = 15;
    const SIGKILL: i32 = 9;
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
        fn geteuid() -> u32;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    enum Protocol {
        Tcp,
        Udp,
    }

    impl Protocol {
        fn as_str(self) -> &'static str {
            match self {
                Self::Tcp => "tcp",
                Self::Udp => "udp",
            }
        }
    }

    #[derive(Debug, Clone)]
    struct Socket {
        protocol: Protocol,
        local_address: String,
        port: u16,
        state: String,
        inode: u64,
        uid: u32,
    }
    #[derive(Debug, Clone)]
    struct ProcessInfo {
        pid: u32,
        parent_pid: Option<u32>,
        command: String,
        executable: String,
        cwd: String,
        uid: u32,
        user: String,
        container_id: Option<String>,
        systemd_unit: Option<String>,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ActionKind {
        External,
        Sigterm,
    }
    #[derive(Debug, Clone)]
    struct ResolutionAction {
        kind: ActionKind,
        program: String,
        args: Vec<String>,
        display: String,
    }
    impl ResolutionAction {
        fn external(program: String, args: Vec<String>, display: String) -> Self {
            Self {
                kind: ActionKind::External,
                program,
                args,
                display,
            }
        }
        fn sigterm(pid: u32) -> Self {
            Self {
                kind: ActionKind::Sigterm,
                program: "kill".to_string(),
                args: vec!["-TERM".to_string(), pid.to_string()],
                display: format!("kill -TERM {pid}  # graceful process stop"),
            }
        }
        fn is_sigterm(&self) -> bool {
            self.kind == ActionKind::Sigterm
        }
    }
    pub fn run(args: &Args) -> Result<(), String> {
        let port = args.port.ok_or_else(|| "a port is required".to_string())?;
        let palette = Palette::new(args.no_color, args.json);
        let sockets = find_sockets(port, args.protocol)?;

        let (processes, socket_owners) = collect_processes();
        let mut matches = Vec::new();
        let mut seen = HashSet::new();

        for socket in sockets {
            let owners = socket_owners
                .get(&socket.inode)
                .map(|pids| {
                    pids.iter()
                        .filter_map(|pid| processes.get(pid))
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let key = (
                socket.protocol,
                socket.inode,
                owners.iter().map(|p| p.pid).collect::<Vec<_>>(),
            );
            if seen.insert(key) {
                matches.push((socket, owners));
            }
        }

        if args.json {
            print_json(port, &matches, &processes);
        } else {
            print_human(port, &matches, &processes, &palette);
        }

        if args.resolve && !matches.is_empty() {
            resolve_interactively(&matches, &palette)?;
        }

        Ok(())
    }

    fn find_sockets(port: u16, filter: ProtocolFilter) -> Result<Vec<Socket>, String> {
        let mut sockets = Vec::new();
        let tables = match filter {
            ProtocolFilter::Tcp => vec![
                ("/proc/net/tcp", Protocol::Tcp),
                ("/proc/net/tcp6", Protocol::Tcp),
            ],
            ProtocolFilter::Udp => vec![
                ("/proc/net/udp", Protocol::Udp),
                ("/proc/net/udp6", Protocol::Udp),
            ],
            ProtocolFilter::All => vec![
                ("/proc/net/tcp", Protocol::Tcp),
                ("/proc/net/tcp6", Protocol::Tcp),
                ("/proc/net/udp", Protocol::Udp),
                ("/proc/net/udp6", Protocol::Udp),
            ],
        };

        for (path, protocol) in tables {
            let contents = fs::read_to_string(path)
                .map_err(|error| format!("could not read {path}: {error}"))?;
            for line in contents.lines().skip(1) {
                if let Some(socket) = parse_socket_line(line, protocol) {
                    if socket.port == port {
                        sockets.push(socket);
                    }
                }
            }
        }
        Ok(sockets)
    }

    fn parse_socket_line(line: &str, protocol: Protocol) -> Option<Socket> {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 12 {
            return None;
        }
        let (address_hex, port_hex) = fields.get(1)?.split_once(':')?;
        let port = u16::from_str_radix(port_hex, 16).ok()?;
        let inode = fields.get(9)?.parse::<u64>().ok()?;
        let uid = fields.get(7)?.parse::<u32>().ok()?;
        let local_address = decode_address(address_hex)?;
        let state = socket_state(fields.get(3)?, protocol);

        Some(Socket {
            protocol,
            local_address,
            port,
            state,
            inode,
            uid,
        })
    }

    fn socket_state(value: &str, protocol: Protocol) -> String {
        if protocol == Protocol::Udp {
            return match value {
                "07" => "UNCONN".to_string(),
                "01" => "ESTABLISHED".to_string(),
                other => format!("0x{other}"),
            };
        }
        match value {
            "01" => "ESTABLISHED".to_string(),
            "02" => "SYN_SENT".to_string(),
            "03" => "SYN_RECV".to_string(),
            "04" => "FIN_WAIT1".to_string(),
            "05" => "FIN_WAIT2".to_string(),
            "06" => "TIME_WAIT".to_string(),
            "07" => "CLOSE".to_string(),
            "08" => "CLOSE_WAIT".to_string(),
            "09" => "LAST_ACK".to_string(),
            "0A" => "LISTEN".to_string(),
            "0B" => "CLOSING".to_string(),
            "0C" => "NEW_SYN_RECV".to_string(),
            other => format!("0x{other}"),
        }
    }

    fn decode_address(value: &str) -> Option<String> {
        if value.len() == 8 {
            let bytes = hex_bytes(value)?;
            return Some(format!(
                "{}.{}.{}.{}",
                bytes[3], bytes[2], bytes[1], bytes[0]
            ));
        }
        if value.len() == 32 {
            let mut output = String::new();
            for (index, chunk) in value.as_bytes().chunks(8).enumerate() {
                let chunk = std::str::from_utf8(chunk).ok()?;
                let bytes = hex_bytes(chunk)?;
                if index > 0 {
                    output.push(':');
                }
                output.push_str(&format!(
                    "{:02x}{:02x}{:02x}{:02x}",
                    bytes[3], bytes[2], bytes[1], bytes[0]
                ));
            }
            return Some(output);
        }
        None
    }

    fn hex_bytes(value: &str) -> Option<Vec<u8>> {
        if value.len() & 1 != 0 {
            return None;
        }
        (0..value.len())
            .step_by(2)
            .map(|index| u8::from_str_radix(&value[index..index + 2], 16).ok())
            .collect()
    }

    fn collect_processes() -> (HashMap<u32, ProcessInfo>, HashMap<u64, Vec<u32>>) {
        let mut processes = HashMap::new();
        let mut socket_owners: HashMap<u64, Vec<u32>> = HashMap::new();
        let entries = match fs::read_dir("/proc") {
            Ok(entries) => entries,
            Err(_) => return (processes, socket_owners),
        };

        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            let Ok(pid) = name.parse::<u32>() else {
                continue;
            };
            let Some(info) = read_process(pid) else {
                continue;
            };

            let fd_path = format!("/proc/{pid}/fd");
            if let Ok(fds) = fs::read_dir(fd_path) {
                for fd in fds.flatten() {
                    if let Ok(target) = fs::read_link(fd.path()) {
                        if let Some(inode) = socket_inode(&target) {
                            let owners = socket_owners.entry(inode).or_default();
                            if !owners.contains(&pid) {
                                owners.push(pid);
                            }
                        }
                    }
                }
            }
            processes.insert(pid, info);
        }
        (processes, socket_owners)
    }

    fn socket_inode(target: &Path) -> Option<u64> {
        let target = target.to_string_lossy();
        let start = target.find("socket:[")? + "socket:[".len();
        let end = target[start..].find(']')? + start;
        target[start..end].parse().ok()
    }

    fn read_process(pid: u32) -> Option<ProcessInfo> {
        let root = PathBuf::from(format!("/proc/{pid}"));
        let status = fs::read_to_string(root.join("status")).ok()?;
        let parent_pid = status_value(&status, "PPid:").and_then(|value| value.parse().ok());
        let uid = status_value(&status, "Uid:")
            .and_then(|value| value.split_whitespace().next())
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        let command = read_cmdline(&root).or_else(|| fs::read_to_string(root.join("comm")).ok())?;
        let executable = fs::read_link(root.join("exe"))
            .ok()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| command.split_whitespace().next().unwrap_or("?").to_string());
        let cwd = fs::read_link(root.join("cwd"))
            .ok()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| "<permission denied>".to_string());
        let cgroup = fs::read_to_string(root.join("cgroup")).unwrap_or_default();
        let container_id = extract_container_id(&cgroup);
        let systemd_unit = extract_systemd_unit(&cgroup);

        Some(ProcessInfo {
            pid,
            parent_pid,
            command,
            executable,
            cwd,
            uid,
            user: username_for_uid(uid),
            container_id,
            systemd_unit,
        })
    }

    fn status_value<'a>(status: &'a str, key: &str) -> Option<&'a str> {
        status
            .lines()
            .find_map(|line| line.strip_prefix(key).map(str::trim))
    }

    fn read_cmdline(root: &Path) -> Option<String> {
        let bytes = fs::read(root.join("cmdline")).ok()?;
        if bytes.is_empty() {
            return None;
        }
        let command = bytes
            .split(|byte| *byte == 0)
            .filter(|part| !part.is_empty())
            .map(|part| String::from_utf8_lossy(part).into_owned())
            .collect::<Vec<_>>()
            .join(" ");
        (!command.is_empty()).then_some(command)
    }

    fn username_for_uid(uid: u32) -> String {
        let Ok(passwd) = fs::read_to_string("/etc/passwd") else {
            return uid.to_string();
        };
        passwd
            .lines()
            .find_map(|line| {
                let fields: Vec<&str> = line.split(':').collect();
                if fields.get(2)?.parse::<u32>().ok()? == uid {
                    Some(fields[0].to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| uid.to_string())
    }

    fn extract_container_id(cgroup: &str) -> Option<String> {
        for raw in cgroup.split(['/', ':', ' ']) {
            let value = raw
                .strip_prefix("docker-")
                .or_else(|| raw.strip_prefix("libpod-"))
                .or_else(|| raw.strip_prefix("crio-"))
                .unwrap_or(raw)
                .trim_end_matches(".scope");
            if value.len() >= 12 && value.chars().all(|character| character.is_ascii_hexdigit()) {
                return Some(value[..12.min(value.len())].to_string());
            }
        }
        None
    }

    fn extract_systemd_unit(cgroup: &str) -> Option<String> {
        cgroup
            .split('/')
            .find(|part| part.ends_with(".service") || part.ends_with(".scope"))
            .map(str::to_string)
    }

    fn parent_chain(pid: u32, processes: &HashMap<u32, ProcessInfo>) -> Vec<String> {
        let mut chain = Vec::new();
        let mut current = Some(pid);
        let mut visited = HashSet::new();
        while let Some(current_pid) = current {
            if !visited.insert(current_pid) || chain.len() >= 8 {
                break;
            }
            let Some(process) = processes.get(&current_pid) else {
                break;
            };
            chain.push(format!(
                "{} ({})",
                short_command(&process.command),
                process.pid
            ));
            current = process.parent_pid;
        }
        chain
    }

    fn short_command(command: &str) -> &str {
        command.split_whitespace().next().unwrap_or(command)
    }

    struct Palette {
        enabled: bool,
    }
    impl Palette {
        fn new(disabled: bool, json: bool) -> Self {
            Self {
                enabled: !disabled
                    && !json
                    && io::stdout().is_terminal()
                    && env::var_os("NO_COLOR").is_none(),
            }
        }
        fn paint(&self, code: &str, value: impl std::fmt::Display) -> String {
            let value = value.to_string();
            if self.enabled {
                format!("{code}{value}\x1b[0m")
            } else {
                value
            }
        }
        fn brand(&self, value: impl std::fmt::Display) -> String {
            self.paint("\x1b[38;5;45;1m", value)
        }
        fn heading(&self, value: impl std::fmt::Display) -> String {
            self.paint("\x1b[38;5;39;1m", value)
        }
        fn label(&self, value: impl std::fmt::Display) -> String {
            self.paint("\x1b[38;5;111;1m", value)
        }
        fn value(&self, value: impl std::fmt::Display) -> String {
            self.paint("\x1b[37;1m", value)
        }
        fn good(&self, value: impl std::fmt::Display) -> String {
            self.paint("\x1b[38;5;82;1m", value)
        }
        fn warn(&self, value: impl std::fmt::Display) -> String {
            self.paint("\x1b[38;5;220;1m", value)
        }
        fn danger(&self, value: impl std::fmt::Display) -> String {
            self.paint("\x1b[38;5;203;1m", value)
        }
        fn muted(&self, value: impl std::fmt::Display) -> String {
            self.paint("\x1b[2;37m", value)
        }
        fn command(&self, value: impl std::fmt::Display) -> String {
            self.paint("\x1b[38;5;114m", value)
        }
        fn rule(&self) -> String {
            self.muted("────────────────────────────────────────────────────────")
        }
    }
    fn print_human(
        port: u16,
        matches: &[(Socket, Vec<ProcessInfo>)],
        processes: &HashMap<u32, ProcessInfo>,
        palette: &Palette,
    ) {
        println!("{}", palette.brand("portopsy"));
        println!("{}", palette.muted(format!("Inspecting port {port}")));
        println!("{}", palette.rule());
        if matches.is_empty() {
            println!(
                "{}",
                palette.good(format!("No socket is using port {port}."))
            );
            println!(
                "{}",
                palette.muted("Try --tcp or --udp to narrow the search.")
            );
            return;
        }
        println!(
            "{}",
            palette.heading(format!("{} socket match(es)", matches.len()))
        );
        for (index, (socket, owners)) in matches.iter().enumerate() {
            if index > 0 {
                println!();
            }
            let endpoint = if socket.local_address.contains(':') {
                format!("[{}]:{}", socket.local_address, socket.port)
            } else {
                format!("{}:{}", socket.local_address, socket.port)
            };
            println!(
                "{}  {}",
                palette.label(format!(
                    "{}://{}",
                    socket.protocol.as_str().to_uppercase(),
                    endpoint
                )),
                if socket.state == "LISTEN" || socket.state == "UNCONN" {
                    palette.good(&socket.state)
                } else {
                    palette.warn(&socket.state)
                }
            );
            println!(
                "  {} {}",
                palette.label(format!("{:<12}", "protocol")),
                palette.value(socket.protocol.as_str().to_uppercase())
            );
            println!(
                "  {} {}",
                palette.label(format!("{:<12}", "endpoint")),
                palette.value(endpoint)
            );
            println!(
                "  {} {}",
                palette.label(format!("{:<12}", "inode")),
                palette.muted(format!("{}  uid {}", socket.inode, socket.uid))
            );
            if owners.is_empty() {
                println!(
                    "  {}",
                    palette.danger("process      <hidden or unavailable>")
                );
                println!(
                    "  {}",
                    palette.warn("Try running with sudo to inspect another user's process.")
                );
                continue;
            }
            let mut printed = HashSet::new();
            for process in owners {
                if !printed.insert(process.pid) {
                    continue;
                }
                println!();
                println!(
                    "  {} {}",
                    palette.label(format!("{:<12}", "process")),
                    palette.value(format!("{} (PID {})", process.command, process.pid))
                );
                println!(
                    "  {} {}",
                    palette.label(format!("{:<12}", "executable")),
                    palette.muted(&process.executable)
                );
                println!(
                    "  {} {}",
                    palette.label(format!("{:<12}", "user")),
                    palette.value(format!("{} (uid {})", process.user, process.uid))
                );
                println!(
                    "  {} {}",
                    palette.label(format!("{:<12}", "cwd")),
                    palette.value(&process.cwd)
                );
                println!(
                    "  {} {}",
                    palette.label(format!("{:<12}", "parents")),
                    palette.muted(parent_chain(process.pid, processes).join(" -> "))
                );
                if let Some(container_id) = &process.container_id {
                    println!(
                        "  {} {}",
                        palette.label(format!("{:<12}", "container")),
                        palette.warn(format!("{} (cgroup)", container_id))
                    );
                }
                if let Some(unit) = &process.systemd_unit {
                    println!(
                        "  {} {}",
                        palette.label(format!("{:<12}", "systemd")),
                        palette.warn(unit)
                    );
                }
                println!();
                println!("  {}", palette.warn("SAFE ACTIONS  (not executed)"));
                for action in actions_for(process) {
                    println!("    {}", palette.command(format!("$ {}", action.display)));
                }
            }
        }
        println!();
        println!("{}", palette.rule());
        println!(
            "{}",
            palette.muted("Read-only by default. Use --resolve for guarded interactive actions.")
        );
    }
    fn actions_for(process: &ProcessInfo) -> Vec<ResolutionAction> {
        let mut actions = Vec::new();
        if let Some(unit) = &process.systemd_unit {
            if current_uid() == 0 {
                actions.push(ResolutionAction::external(
                    "systemctl".to_string(),
                    vec!["stop".to_string(), unit.clone()],
                    format!("systemctl stop {unit}"),
                ));
            } else {
                actions.push(ResolutionAction::external(
                    "sudo".to_string(),
                    vec!["systemctl".to_string(), "stop".to_string(), unit.clone()],
                    format!("sudo systemctl stop {unit}"),
                ));
            }
        }
        if let Some(container_id) = &process.container_id {
            actions.push(ResolutionAction::external(
                "docker".to_string(),
                vec!["stop".to_string(), container_id.clone()],
                format!("docker stop {container_id}"),
            ));
            actions.push(ResolutionAction::external(
                "podman".to_string(),
                vec!["stop".to_string(), container_id.clone()],
                format!("podman stop {container_id}"),
            ));
        }
        actions.push(ResolutionAction::sigterm(process.pid));
        actions
    }
    fn interactive_terminal_available(stdout: bool, stdin: bool) -> bool {
        stdout && stdin
    }

    fn resolve_interactively(
        matches: &[(Socket, Vec<ProcessInfo>)],
        palette: &Palette,
    ) -> Result<(), String> {
        if !interactive_terminal_available(io::stdout().is_terminal(), io::stdin().is_terminal()) {
            return Err(palette.danger(
                "--resolve requires interactive stdout and stdin; it cannot run in a pipe, CI job, or JSON mode",
            ));
        }
        let mut candidates: Vec<&ProcessInfo> = Vec::new();
        let mut seen_pids = HashSet::new();
        for (_, owners) in matches {
            for process in owners {
                if seen_pids.insert(process.pid) {
                    candidates.push(process);
                }
            }
        }
        if candidates.is_empty() {
            return Err(
                palette.danger("no visible process is available to resolve; try running with sudo")
            );
        }
        println!();
        println!("{}", palette.heading("RESOLUTION MODE"));
        println!(
            "{}",
            palette.warn("This can stop a process, service, or container.")
        );
        let process_index = if candidates.len() == 1 {
            0
        } else {
            println!();
            println!("{}", palette.heading("Select a process"));
            for (index, process) in candidates.iter().enumerate() {
                println!(
                    "  {} {}  {}",
                    palette.label(format!("[{}]", index + 1)),
                    palette.value(format!("PID {}", process.pid)),
                    palette.muted(&process.command)
                );
                println!(
                    "      {} {}",
                    palette.label(format!("{:<10}", "cwd")),
                    palette.muted(&process.cwd)
                );
            }
            let prompt = palette.label("Process [0 to cancel]:");
            let Some(index) = prompt_index(&prompt, candidates.len())? else {
                println!("{}", palette.muted("Cancelled. No action was executed."));
                return Ok(());
            };
            index
        };
        let process = candidates[process_index];
        let actions = actions_for(process);
        let systemd_action = actions
            .iter()
            .find(|action| {
                action.program == "systemctl"
                    || (action.program == "sudo"
                        && action.args.first().map(String::as_str) == Some("systemctl"))
            })
            .filter(|action| command_available(&action.program))
            .cloned();
        let container_action = actions
            .iter()
            .find(|action| action.program == "docker" && command_available(&action.program))
            .or_else(|| {
                actions
                    .iter()
                    .find(|action| action.program == "podman" && command_available(&action.program))
            })
            .cloned();
        let sigterm_action = actions.iter().find(|action| action.is_sigterm()).cloned();
        println!();
        println!(
            "{}",
            palette.heading(format!("Target  PID {}", process.pid))
        );
        println!(
            "  {} {}",
            palette.label(format!("{:<10}", "command")),
            palette.value(&process.command)
        );
        println!(
            "  {} {}",
            palette.label(format!("{:<10}", "cwd")),
            palette.muted(&process.cwd)
        );
        println!();
        println!("{}", palette.heading("Choose an action"));
        print_menu_action(
            palette,
            "[1]",
            "Stop systemd service",
            systemd_action.as_ref(),
        );
        print_menu_action(palette, "[2]", "Stop container", container_action.as_ref());
        print_menu_action(palette, "[3]", "Send SIGTERM", sigterm_action.as_ref());
        println!("  {} {}", palette.label("[4]"), palette.muted("Cancel"));
        let prompt = palette.label("Choice [1-4, 0 to cancel]:");
        let Some(action_index) = prompt_index(&prompt, 4)? else {
            println!("{}", palette.muted("Cancelled. No action was executed."));
            return Ok(());
        };
        if action_index == 3 {
            println!("{}", palette.muted("Cancelled. No action was executed."));
            return Ok(());
        }
        let action = match action_index {
            0 => systemd_action,
            1 => container_action,
            2 => sigterm_action,
            _ => None,
        };
        let Some(action) = action else {
            println!(
                "{}",
                palette.warn("That action is unavailable for this process.")
            );
            return Ok(());
        };
        println!();
        println!(
            "{} {}",
            palette.warn("About to execute:"),
            palette.command(&action.display)
        );
        let confirm_prompt = palette.warn("Type 'yes' to continue, anything else to cancel:");
        if !confirm_yes(&confirm_prompt)? {
            println!("{}", palette.muted("Cancelled. No action was executed."));
            return Ok(());
        }
        let current = read_process(process.pid)
            .ok_or_else(|| format!("PID {} no longer exists; nothing was executed", process.pid))?;
        if current.command != process.command || current.executable != process.executable {
            return Err(format!(
                "PID {} changed since inspection; rerun portopsy before resolving it",
                process.pid
            ));
        }
        execute_action(&action, process, palette)?;
        println!(
            "{} {}",
            palette.good("Executed successfully:"),
            palette.command(&action.display)
        );
        println!(
            "{}",
            palette.muted("Run portopsy again to verify that the port is free.")
        );
        Ok(())
    }
    fn execute_action(
        action: &ResolutionAction,
        process: &ProcessInfo,
        palette: &Palette,
    ) -> Result<(), String> {
        if action.is_sigterm() {
            execute_sigterm(process, palette)
        } else {
            execute_external(action)
        }
    }
    fn execute_external(action: &ResolutionAction) -> Result<(), String> {
        if action.program == "sudo" {
            let status = Command::new(&action.program)
                .args(&action.args)
                .status()
                .map_err(|error| {
                    if error.kind() == io::ErrorKind::PermissionDenied {
                        permission_guidance()
                    } else {
                        format!("could not execute {}: {error}", action.program)
                    }
                })?;
            if status.success() {
                return Ok(());
            }
            return Err(permission_guidance());
        }
        let output = Command::new(&action.program)
            .args(&action.args)
            .output()
            .map_err(|error| {
                if error.kind() == io::ErrorKind::PermissionDenied {
                    permission_guidance()
                } else {
                    format!("could not execute {}: {error}", action.program)
                }
            })?;
        if output.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        let permission_text = stderr.to_ascii_lowercase();
        if output.status.code() == Some(130)
            || output.status.code() == Some(137)
            || permission_text.contains("permission denied")
            || permission_text.contains("operation not permitted")
            || permission_text.contains("eperm")
        {
            return Err(permission_guidance());
        }
        Err(format!(
            "{} exited with status {}{}",
            action.program,
            output.status,
            if stderr.trim().is_empty() {
                String::new()
            } else {
                format!(": {}", stderr.trim())
            }
        ))
    }

    fn execute_sigterm(process: &ProcessInfo, palette: &Palette) -> Result<(), String> {
        let pid = process.pid;
        send_signal(pid, SIGTERM)?;
        println!(
            "{}",
            palette.muted("SIGTERM sent; waiting up to 1.5 seconds for the process to exit...")
        );
        if !wait_for_exit(pid, std::time::Duration::from_millis(1500)) {
            return Ok(());
        }
        let prompt = palette.warn(format!(
            "Process did not exit after SIGTERM. Send SIGKILL (kill -9 {pid})? [y/N]"
        ));
        if !confirm_yes_no(&prompt)? {
            println!(
                "{}",
                palette.muted("Process is still running. No SIGKILL was sent.")
            );
            return Ok(());
        }
        let current = read_process(pid).ok_or_else(|| {
            format!("PID {pid} exited before SIGKILL; no further action was needed")
        })?;
        if current.command != process.command || current.executable != process.executable {
            return Err(format!("PID {pid} changed before SIGKILL; rerun portopsy"));
        }
        send_signal(pid, SIGKILL)?;
        if wait_for_exit(pid, std::time::Duration::from_millis(500)) {
            return Err(format!("PID {pid} is still running after SIGKILL"));
        }
        println!(
            "{}",
            palette.good(format!("Process {pid} exited after SIGKILL."))
        );
        Ok(())
    }

    fn print_menu_action(
        palette: &Palette,
        number: &str,
        label: &str,
        action: Option<&ResolutionAction>,
    ) {
        println!("  {} {}", palette.label(number), palette.value(label));
        if let Some(action) = action {
            println!("      {}", palette.muted(format!("$ {}", action.display)));
        } else {
            println!("      {}", palette.muted("unavailable for this process"));
        }
    }

    fn prompt_index(prompt: &str, maximum: usize) -> Result<Option<usize>, String> {
        loop {
            print!("{prompt} ");
            io::stdout().flush().map_err(|error| error.to_string())?;
            let mut input = String::new();
            let bytes = io::stdin()
                .read_line(&mut input)
                .map_err(|error| format!("could not read input: {error}"))?;
            if bytes == 0 {
                return Ok(None);
            }
            let input = input.trim();
            let Ok(choice) = input.parse::<usize>() else {
                println!("Enter a number from 0 to {maximum}.");
                continue;
            };
            if choice == 0 {
                return Ok(None);
            }
            if choice <= maximum {
                return Ok(Some(choice - 1));
            }
            println!("Enter a number from 0 to {maximum}.");
        }
    }

    fn confirm_yes_no(prompt: &str) -> Result<bool, String> {
        print!("{prompt} ");
        io::stdout().flush().map_err(|error| error.to_string())?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|error| format!("could not read input: {error}"))?;
        Ok(matches!(input.trim(), "y" | "Y"))
    }

    fn confirm_yes(prompt: &str) -> Result<bool, String> {
        print!("{prompt} ");
        io::stdout().flush().map_err(|error| error.to_string())?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|error| format!("could not read input: {error}"))?;
        Ok(input.trim() == "yes")
    }

    fn current_uid() -> u32 {
        unsafe { geteuid() }
    }

    fn send_signal(pid: u32, signal: i32) -> Result<(), String> {
        let result = unsafe { kill(pid as i32, signal) };
        if result == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::PermissionDenied {
            return Err(permission_guidance());
        }
        Err(format!(
            "could not send signal {signal} to PID {pid}: {error}"
        ))
    }

    fn process_alive(pid: u32) -> bool {
        Path::new(&format!("/proc/{pid}")).exists()
    }

    fn wait_for_exit(pid: u32, timeout: std::time::Duration) -> bool {
        let deadline = std::time::Instant::now() + timeout;
        while std::time::Instant::now() < deadline {
            if !process_alive(pid) {
                return false;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        process_alive(pid)
    }

    fn permission_guidance() -> String {
        "Action failed due to permission constraints. Try running with elevated privileges:\n  $ sudo portopsy --resolve".to_string()
    }

    fn command_available(program: &str) -> bool {
        Command::new(program)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn print_json(
        port: u16,
        matches: &[(Socket, Vec<ProcessInfo>)],
        processes: &HashMap<u32, ProcessInfo>,
    ) {
        let mut output = format!("{{\"port\":{port},\"matches\":[");
        for (index, (socket, owners)) in matches.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            output.push_str(&format!(
                "{{\"protocol\":\"{}\",\"local_address\":\"{}\",\"port\":{},\"state\":\"{}\",\"inode\":{},\"uid\":{},\"owners\":[",
                socket.protocol.as_str(),
                json_escape(&socket.local_address),
                socket.port,
                json_escape(&socket.state),
                socket.inode,
                socket.uid
            ));
            for (owner_index, process) in owners.iter().enumerate() {
                if owner_index > 0 {
                    output.push(',');
                }
                let parents = parent_chain(process.pid, processes).join(" -> ");
                output.push_str(&format!(
                    "{{\"pid\":{},\"command\":\"{}\",\"executable\":\"{}\",\"cwd\":\"{}\",\"uid\":{},\"user\":\"{}\",\"parent_chain\":\"{}\"",
                    process.pid,
                    json_escape(&process.command),
                    json_escape(&process.executable),
                    json_escape(&process.cwd),
                    process.uid,
                    json_escape(&process.user),
                    json_escape(&parents)
                ));
                if let Some(container_id) = &process.container_id {
                    output.push_str(&format!(
                        ",\"container_id\":\"{}\"",
                        json_escape(container_id)
                    ));
                } else {
                    output.push_str(",\"container_id\":null");
                }
                if let Some(unit) = &process.systemd_unit {
                    output.push_str(&format!(",\"systemd_unit\":\"{}\"", json_escape(unit)));
                } else {
                    output.push_str(",\"systemd_unit\":null");
                }
                output.push_str(",\"actions\":[");
                for (action_index, action) in actions_for(process).iter().enumerate() {
                    if action_index > 0 {
                        output.push(',');
                    }
                    output.push_str(&format!("\"{}\"", json_escape(&action.display)));
                }
                output.push_str("]}");
            }
            output.push_str("]}");
        }
        output.push_str("]}");
        println!("{output}");
    }

    fn json_escape(value: &str) -> String {
        let mut escaped = String::with_capacity(value.len());
        for character in value.chars() {
            match character {
                '"' => escaped.push_str("\\\""),
                '\\' => escaped.push_str("\\\\"),
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                character if character.is_control() => {
                    escaped.push_str(&format!("\\u{:04x}", character as u32))
                }
                character => escaped.push(character),
            }
        }
        escaped
    }

    #[cfg(test)]
    mod tests {
        use super::{decode_address, extract_container_id, parse_socket_line, Protocol};

        #[test]
        fn decodes_ipv4_proc_address() {
            assert_eq!(decode_address("0100007F").unwrap(), "127.0.0.1");
            assert_eq!(decode_address("00000000").unwrap(), "0.0.0.0");
        }

        #[test]
        fn parses_tcp_socket_line() {
            let line = "  12: 0100007F:0BB8 00000000:0000 0A 00000000:00000000 00:00000000 00000000   1000        0 424242 1 0000000000000000 100 0 0 10 0";
            let socket = parse_socket_line(line, Protocol::Tcp).unwrap();
            assert_eq!(socket.port, 3000);
            assert_eq!(socket.local_address, "127.0.0.1");
            assert_eq!(socket.state, "LISTEN");
            assert_eq!(socket.inode, 424242);
        }

        #[test]
        fn detects_container_ids_from_cgroups() {
            let cgroup = "0::/user.slice/user-1000.slice/user@1000.service/user.slice/libpod-0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef.scope";
            assert_eq!(
                extract_container_id(cgroup).as_deref(),
                Some("0123456789ab")
            );
        }

        #[test]
        fn creates_non_shell_sigterm_action() {
            let process = super::ProcessInfo {
                pid: 4242,
                parent_pid: None,
                command: "node server.js".to_string(),
                executable: "/usr/bin/node".to_string(),
                cwd: "/tmp/app".to_string(),
                uid: 1000,
                user: "tester".to_string(),
                container_id: None,
                systemd_unit: None,
            };
            let actions = super::actions_for(&process);
            let action = actions.last().unwrap();
            assert_eq!(action.program, "kill");
            assert_eq!(action.args, vec!["-TERM", "4242"]);
            assert!(action.display.contains("kill -TERM 4242"));
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    use crate::cli::Args;

    pub fn run(args: &Args) -> Result<(), String> {
        let _ = (
            &args.port,
            &args.protocol,
            &args.json,
            &args.resolve,
            &args.no_color,
        );
        Err(
            "this build currently supports Linux only; run it inside WSL or a Linux host"
                .to_string(),
        )
    }
}

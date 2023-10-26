use std::io::Write;
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

macro_rules! parse_arg {
    ($args:expr, $short:expr, $long:expr, $var:expr, $msg:expr) => {
        if let Some(index) = $args.iter().position(|arg| arg == $short || arg == $long) {
            if let Ok(parsed_num) = $args[index + 1].parse() {
                $var = parsed_num;
            } else {
                eprintln!($msg);
                std::process::exit(1);
            }
        }
    };
}

fn is_port_open(ip: &str, port: u16, timeout: u64) -> bool
{
    let ip_addr: Result<IpAddr, _> = ip.parse();
    if let Ok(ip_addr) = ip_addr {
        let socket_addr = SocketAddr::new(ip_addr, port);
        TcpStream::connect_timeout(&socket_addr, Duration::from_secs(timeout)).is_ok()
    } else {
        eprintln!("Could not parse ip: {}", ip);
        std::process::exit(1);
    }
}

fn dns_resolve(hostname: &str) -> String
{
    let socket_addrs = (hostname, 0).to_socket_addrs();

    if let Ok(mut addrs) = socket_addrs {
        if let Some(addr) = addrs.next() {
            return addr.ip().to_string();
        }
    } else {
        eprintln!("Hostname resolve failed");
        std::process::exit(1);
    }

    String::default()
}

fn set_terminal_title(title: &str)
{
    // i hate microsoft
    #[cfg(target_os = "windows")]
    {
        use winapi::um::wincon::SetConsoleTitleW;
        use winapi::um::winnt::WCHAR;

        let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        let _result = unsafe { SetConsoleTitleW(wide_title.as_ptr() as *const WCHAR) };
    }

    #[cfg(not(target_os = "windows"))]
    {
        print!("\x1B]2;{}\x07", title);
    }
}

fn print_colored_text(text: &str, color: Color)
{
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut color_spec = ColorSpec::new();

    color_spec.set_fg(Some(color));
    stdout.set_color(&color_spec).unwrap();

    write!(stdout, "{}", text).unwrap();
    stdout.reset().unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!(
            "Usage: {} <IP address / hostname> <Port>\n
            \r[-t, --timeout] in seconds\n
            \rDefault timeout is 2 seconds\n",
            args[0]
        );
        std::process::exit(0);
    }

    let target = dns_resolve(&args[1]);
    let port = match args[2].parse::<u16>() {
        Ok(parsed_port) => parsed_port,
        Err(e) => {
            eprintln!("Failed to parse port: {}", e);
            std::process::exit(1);
        }
    };
    let mut timeout = 2;

    parse_arg!(&args, "-t", "--timeout", timeout, "-t <timeout in seconds>");

    set_terminal_title(&format!("Probing {} on port {}", target, port));
    loop {
        let start_time = Instant::now();
        if is_port_open(&target, port, timeout) {
            let end_time = Instant::now();
            let duration = end_time.duration_since(start_time).as_millis();

            let color = match duration {
                0..=99 => Color::Rgb(6, 156, 86),
                100..=149 => Color::Rgb(255, 152, 14),
                _ => Color::Rgb(211, 33, 44),
            };
            
            print_colored_text("Connected ", color);
            print!("to ");
            print_colored_text(&target, color);
            print!(" on port ");
            print_colored_text(&port.to_string(), color);
            print!(" ms: ");
            print_colored_text(&duration.to_string(), color);
            print!("\n")
        }
        else {
            let color = Color::Rgb(211, 33, 44);
            print_colored_text("Failed ", color);
            print!("to connect to ");
            print_colored_text(&target, color);
            print!(" on port ");
            print_colored_text(&port.to_string(), color);
            print!("\n")
        }
        std::thread::sleep(Duration::from_secs(timeout));
    }
}
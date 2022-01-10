use std::net::{SocketAddr, TcpListener, TcpStream, Shutdown};

fn main() {
    let (listen_on, forward_to, buf_size, extra_debug) = {
        use clap::{App, Arg};
        let a = App::new("Port Proxy Daemon")
            .author("SirAlador")
            .version("0.1.0")
            .about("Listens on a TCP port and proxies all traffic to a destination")
            .arg(Arg::new("listen_on")
                 //.index(1)
                 .short('l')
                 .long("listen-on")
                 .takes_value(true)
                 .required(true)
                 .help("The [ADDRESS:]PORT to listen on"))
            .arg(Arg::new("forward_to")
                 //.index(2)
                 .short('f')
                 .long("forward-to")
                 .takes_value(true)
                 .required(true)
                 .help("The ADDRESS:PORT to forward to"))
            .arg(Arg::new("buf_size")
                 .short('b')
                 .long("buffer-size")
                 .takes_value(true)
                 .help("The size for each communication buffer. Each active connection has two communication buffers"))
            .arg(Arg::new("extra_debug")
                 .short('v')
                 .long("verbose")
                 .help("Print MOAR stuff!")
                 .takes_value(false))
            .get_matches();

        let mut listen_on = a.value_of("listen_on").unwrap().to_string();
        if !listen_on.contains(":") { listen_on = "0.0.0.0:".to_string() + &listen_on; }
        let listen_on: SocketAddr = listen_on.parse().expect("Your --listen-to address was not a valid socket address");
        let forward_to: SocketAddr = a.value_of("forward_to").unwrap().parse().expect("Your --forward-to address was not a valid socket address");
        let buf_size: usize = a.value_of("buf_size").map(|d| d.parse().expect("Your --buffer-size was not a valid number")).unwrap_or(1024 * 1024 / 2);
        let extra_debug: bool = a.is_present("extra_debug");

        (listen_on, forward_to, buf_size, extra_debug)
    };

    let listener = TcpListener::bind(listen_on).expect("Failed to bind to listen_to address");
    let mut id = 0;
    loop {
        let conn = listener.accept();
        match conn {
            Ok((conn, from_addr)) => {
                let to_addr = forward_to.clone();
                socket(conn, from_addr, to_addr, buf_size, id, extra_debug);
                id = id + 1;
            },
            Err(err) => {
                eprintln!("Socket listener died. Aborting.\n{}", err);
                std::process::exit(0);
            },
        }
        println!("Socket loop!");
    }
}

fn socket(a: TcpStream, a_addr: SocketAddr, b_addr: SocketAddr, buf_size: usize, my_id: usize, extra_debug: bool) {
    println!("New socket: {} with id {}", a_addr, my_id);
    match TcpStream::connect(b_addr) {
        Ok(b) => {
            let from_client = match a.try_clone() { Ok(s) => s, Err(err) => { eprintln!("Failed to clone a socket from_client\n{}", err); return; } };
            let to_server = match b.try_clone() { Ok(s) => s, Err(err) => { eprintln!("Failed to clone a socket to_server\n{}", err); return; } };
            std::thread::spawn(move || { socket_transport(from_client, to_server, buf_size, my_id, extra_debug); });
            let from_server = b;
            let to_client = a;
            std::thread::spawn(move || { socket_transport(from_server, to_client, buf_size, my_id, extra_debug); });
        },
        Err(err) => {
            eprintln!("Failed to connect to destination for socket {}\n{}", my_id, err);
        }
    }
}
fn socket_transport(mut a: TcpStream, mut b: TcpStream, buf_size: usize, my_id: usize, extra_debug: bool) {
    let mut buf = vec![0u8; buf_size];
    use std::io::{Read, Write};
    'outer: while let Ok(bytes_read) = a.read(&mut buf) {
        if bytes_read == 0 { 
            if extra_debug { println!("a Transport died {}", my_id); }
            break 'outer;
        }
        let mut total_written = 0;
        while total_written < bytes_read {
            match b.write(&buf[total_written..bytes_read]){
                Ok(bytes_written) => { total_written += bytes_written; }
                Err(_) => {
                    if extra_debug { println!("a Transport died {} 2", my_id); }
                    break 'outer;
                }
            }
        }
    }
    if let Err(err) = a.shutdown(Shutdown::Read) { if extra_debug { eprintln!("Error occured while shutting down read for a transport with id {}\n{}", my_id, err); } }
    if let Err(err) = b.shutdown(Shutdown::Write) { if extra_debug { eprintln!("Error occured while shutting down read for a transport with id {}\n{}", my_id, err); } }
}

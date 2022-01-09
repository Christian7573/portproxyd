use std::net::{SocketAddr, TcpListener, TcpStream};

fn main() {
    let (listen_on, forward_to, buf_size) = {
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
            .get_matches();

        let mut listen_on = a.value_of("listen_on").unwrap().to_string();
        if !listen_on.contains(":") { listen_on = "0.0.0.0:".to_string() + &listen_on; }
        let listen_on: SocketAddr = listen_on.parse().expect("Your --listen-to address was not a valid socket address");
        let forward_to: SocketAddr = a.value_of("forward_to").unwrap().parse().expect("Your --forward-to address was not a valid socket address");
        let buf_size: usize = a.value_of("buf_size").map(|d| d.parse().expect("Your --buffer-size was not a valid number")).unwrap_or(1024 * 1024 / 2);

        (listen_on, forward_to, buf_size)
    };

    let listener = TcpListener::bind(listen_on).expect("Failed to bind to listen_to address");
    let mut id = 0;
    loop {
        let conn = listener.accept();
        match conn {
            Ok((conn, from_addr)) => {
                let to_addr = forward_to.clone();
                socket(conn, from_addr, to_addr, buf_size, id);
                id = id + 1;
            },
            Err(err) => {
                eprintln!("Socket listener died. Aborting.\n{}", err);
                std::process::exit(0);
            },
        }
    }
}

fn socket(a: TcpStream, a_addr: SocketAddr, b_addr: SocketAddr, buf_size: usize, my_id: usize) {
    println!("New socket: {} with id {}", a_addr, my_id);
    match TcpStream::connect(b_addr) {
        Ok(b) => {
            let from_client = match a.try_clone() { Ok(s) => s, Err(err) => { eprintln!("Failed to clone a socket from_client\n{}", err); return; } };
            let to_server = match b.try_clone() { Ok(s) => s, Err(err) => { eprintln!("Failed to clone a socket to_server\n{}", err); return; } };
            std::thread::spawn(move || { socket_transport(from_client, to_server, buf_size, my_id); });
            let from_server = b;
            let to_client = a;
            std::thread::spawn(move || { socket_transport(from_server, to_client, buf_size, my_id); });
        },
        Err(err) => {
            eprintln!("Failed to connect to destination for socket {}\n{}", my_id, err);
        }
    }
}
fn socket_transport(mut a: TcpStream, mut b: TcpStream, buf_size: usize, _my_id: usize) {
    let mut buf = vec![0u8; buf_size];
    use std::io::{Read, Write};
    while let Ok(bytes_read) = a.read(&mut buf) {
        let mut total_written = 0;
        while total_written < bytes_read {
            match b.write(&buf[total_written..]){
                Ok(bytes_written) => { total_written += bytes_written; }
                Err(_) => { return }
            }
        }
    }
}

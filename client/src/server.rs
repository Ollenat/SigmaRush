use std::io::{Read, Write, stdin};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

pub const SERVER_ADDRESS: &str = "127.0.0.1";
pub const SERVER_PORT: u16 = 8080;
pub const MAX_PLAYERS: usize = 2;

pub fn run_server() -> std::io::Result<()> {
    let listener = TcpListener::bind((SERVER_ADDRESS, SERVER_PORT))?;
    listener.set_nonblocking(true)?;

    println!("[SERVER]: Listening on {}:{}", SERVER_ADDRESS, SERVER_PORT);

    let connections = Arc::new(Mutex::new(Vec::new()));
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    let listener_thread = spawn_listener_thread(
        listener,
        Arc::clone(&connections),
        Arc::clone(&shutdown_flag),
    );

    wait_for_exit_command(&shutdown_flag);
    println!("[SERVER]: Shutting down...");
    shutdown_connections(&connections);

    listener_thread.join().expect("Listener thread panicked");

    println!("[SERVER]: Server stopped.");
    Ok(())
}

fn spawn_listener_thread(
    listener: TcpListener,
    connections: Arc<Mutex<Vec<TcpStream>>>,
    shutdown_flag: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while !shutdown_flag.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, addr)) => {
                    if connections.lock().unwrap().len() >= MAX_PLAYERS {
                        println!("[SERVER]: Max players reached, rejecting {}", addr);
                        let _ = stream.shutdown(Shutdown::Both);
                        continue;
                    }

                    let conn_clone = Arc::clone(&connections);
                    thread::spawn(move || {
                        if let Err(e) = handle_client(stream, conn_clone) {
                            eprintln!("[SERVER]: Error handling client: {}", e);
                        }
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    eprintln!("[SERVER]: Accept error: {}", e);
                }
            }
        }
        println!("[SERVER]: Listener stopped.");
    })
}

/// Yields until the user types "exit" in the console
fn wait_for_exit_command(shutdown_flag: &Arc<AtomicBool>) {
    println!("[SERVER]: Type 'exit' to shut down the server.");
    let mut input = String::new();
    loop {
        if stdin().read_line(&mut input).is_ok() {
            if input.trim().eq_ignore_ascii_case("exit") {
                shutdown_flag.store(true, Ordering::Relaxed);
                break;
            }
        }

        input.clear();
    }
}

fn shutdown_connections(connections: &Arc<Mutex<Vec<TcpStream>>>) {
    let mut conn_list = connections.lock().unwrap();
    for stream in conn_list.iter() {
        stream
            .shutdown(Shutdown::Both)
            .expect("Failed to shutdown stream");
    }
    conn_list.clear();
    println!("[SERVER] Disconnected all clients.");
}

fn handle_client(
    mut stream: TcpStream,
    connections: Arc<Mutex<Vec<TcpStream>>>,
) -> std::io::Result<()> {
    let peer = stream.peer_addr()?;
    println!("[SERVER]: New client connected: {}", peer);

    {
        let mut conn_list = connections.lock().unwrap();
        conn_list.push(stream.try_clone()?);
    }
    let mut buffer = [0; 1024];
    let mut data: Vec<u8> = Vec::new();
    'echo: loop {
        data.clear();
        buffer.fill(0); // Clear the buffer
        'read_data: loop {
            // Wait for data to be available using non-blocking read
            let bytes_read = match stream.read(&mut buffer) {
                Ok(0) => break 'echo, // Client has closed the connection
                Ok(n) => n,
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                    continue 'read_data;
                }
                Err(e) => return Err(e),
            };
            data.extend_from_slice(&buffer[..bytes_read]);
            if bytes_read < buffer.len() {
                break 'read_data;
            }
        }

        if data.is_empty() {
            thread::sleep(Duration::from_millis(50));
            continue;
        }
        println!("Received data: {}", String::from_utf8_lossy(&data).trim());
        stream.write_all(&data)?;
        println!("Sent data back to client");
    }

    stream.shutdown(std::net::Shutdown::Both)?;
    let mut conn_list = connections.lock().unwrap();
    conn_list.retain(|connection| {
        connection.peer_addr().expect("Failed to get peer address")
            != stream.peer_addr().expect("Failed to get peer address")
    });
    println!("Client disconnected: {}", stream.peer_addr()?);

    Ok(())
}

use crate::server::{SERVER_ADDRESS, SERVER_PORT};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;

pub fn connect_to_server() -> std::io::Result<()> {
    let mut stream = TcpStream::connect((SERVER_ADDRESS, SERVER_PORT))?;
    println!("Connected to server at {}:{}", SERVER_ADDRESS, SERVER_PORT);

    ctrlc::set_handler({
        let stream = stream.try_clone()?;
        move || {
            stream
                .shutdown(std::net::Shutdown::Both)
                .expect("Failed to shutdown stream");
        }
    })
    .expect("Error setting Ctrl-C handler");

    // Thread to read user input and send to channel
    let _input_handle = thread::spawn({
        let mut stream = stream.try_clone()?;
        move || {
            let mut buf = String::new();
            loop {
                if std::io::stdin().read_line(&mut buf).is_ok() {
                    stream
                        .write_all(buf.as_bytes())
                        .expect("Failed to send message");
                }
                buf.clear(); // Clear buffer for next input
            }
        }
    });

    // Main thread: read from server and print
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            println!("Stream shut down. Exiting.");
            break;
        }
        println!(
            "Received from server: {}",
            String::from_utf8_lossy(&buffer[..bytes_read]).trim()
        );
    }

    Ok(())
}

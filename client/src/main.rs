mod client;
mod server;

use client::connect_to_server;
use server::run_server;

fn main() -> std::io::Result<()> {
    // Choose to host or connect
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "host" {
        run_server()?;
    } else {
        connect_to_server()?;
    }

    Ok(())
}

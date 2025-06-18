# Echo server
I created this echo server to learn about std net and networking in general. It is a simple TCP echo server that listens on a specified port and echoes back any data it receives. It works on localhost, not testen on a remote server.

# Usage
For now, there is a client that can act as a client or a listen server. To run it, clone the repositiory and
```bash
cd client
cargo run -- host
```
This will start the listen server.

To run the client, open another terminal and run:
```bash
cd client
cargo run
```
This will start the client.

## Intentions
I want to learn more about networking in Rust, so I start with std net and then move to tokio and async.
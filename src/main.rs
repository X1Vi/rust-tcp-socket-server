use std::fmt::Error;
use std::net::{TcpListener, TcpStream};
use std::io::{ErrorKind, Write};
use std::thread;
use std::io; // Import the io module

const CLIENTS_LIMIT: usize = 10;
const NONE: Option<TcpStream> = None;
static mut CLIENTS: [Option<TcpStream>; CLIENTS_LIMIT] = [NONE; CLIENTS_LIMIT];
static mut CLIENT_INDEX: usize = 0;

fn handle_client(mut stream: &TcpStream) {
    stream.write(b"Welcome to the server!\n").unwrap();
}

fn send_commands_to_sockets() {
    loop {
        unsafe {
            // Check if there are any clients
            if CLIENT_INDEX > 0 {
                println!("Existing sockets:");

                // Print the existing sockets
                for (index, socket) in CLIENTS.iter().enumerate() {
                    if let Some(stream) = socket {
                        println!("Socket {}: {:?}", index, stream.peer_addr());
                    }
                }

                // Ask the user to select a socket
                print!("Select a socket index to send a command: ");
                io::stdout().flush().unwrap(); // Flush stdout to ensure the prompt appears

                // Read user input for the socket index
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Failed to read line");

                // Parse the input to get the index
                if let Ok(selected_index) = input.trim().parse::<usize>() {
                    if selected_index < CLIENT_INDEX {
                        // Safely unwrap the selected socket
                        if let Some(selected_socket) = &mut CLIENTS[selected_index] {
                            // Ask for the command to send
                            print!("Enter command to send: ");
                            io::stdout().flush().unwrap(); // Flush stdout to ensure the prompt appears

                            let mut command_input = String::new();
                            io::stdin().read_line(&mut command_input).expect("Failed to read line");

                            // Send the command to the selected socket
                            if let Err(e) = selected_socket.write(command_input.as_bytes()) {
                                eprintln!("Failed to send command: {}", e);
                            } else {
                                println!("Command sent to socket {}: {:?}", selected_index, selected_socket.peer_addr());
                            }
                        } else {
                            println!("No socket found at index {}", selected_index);
                        }
                    } else {
                        println!("Invalid index selected.");
                    }
                } else {
                    println!("Failed to parse input.");
                }
            } else {
                println!("No active sockets.");
            }
        }
        thread::sleep(std::time::Duration::from_secs(1)); // Sleep for a short duration to prevent busy waiting
    }
}

fn main() {
    let listener_thread = thread::spawn(|| {
        if let Err(e) = listen_to_active_connections() {
            eprintln!("Listener thread error: {}", e);
        }
        () // Explicitly return the unit type
    });

    // Spawn a separate thread for sending commands
    let command_thread = thread::spawn(|| {
        send_commands_to_sockets();
    });

    // Wait for the listener thread to finish
    listener_thread.join().unwrap();
    // Wait for the command thread to finish (this will not happen since it loops indefinitely)
    command_thread.join().unwrap();
}

fn listen_to_active_connections() -> Result<(), io::Error> {
    let listener = TcpListener::bind("0.0.0.0:8002")?; // Bind to all interfaces on port 8002
    println!("Waiting for connections...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Connection established!");
                handle_client(&stream);
                unsafe {
                    if CLIENT_INDEX < CLIENTS_LIMIT {
                        CLIENTS[CLIENT_INDEX] = Some(stream);
                        CLIENT_INDEX += 1;
                    } else {
                        println!("Client limit reached.");
                    }
                }
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(()) // Indicate that the function completed successfully
}

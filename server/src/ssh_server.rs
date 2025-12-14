// ssh_server.rs
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::env;

pub async fn start_ssh_server() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:2222")?;
    println!("SSH Forum Server listening on port 2222");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("Connection error: {}", e),
        }
    }
    Ok(())
}

fn handle_client(mut stream: std::net::TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    // Simple SSH-like protocol
    stream.write_all(b"Welcome to Arch Forum SSH Server\r\n")?;
    stream.write_all(b"Arch Linux verification required...\r\n")?;
    
    // Check if client is running Arch Linux (simplified)
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    let input = String::from_utf8_lossy(&buffer[..bytes_read]);
    
    if input.contains("arch") || input.contains("linux") {
        stream.write_all(b"Arch Linux verified! Welcome.\r\n")?;
        stream.write_all(b"\r\n=== ARCH FORUM ===\r\n")?;
        stream.write_all(b"Commands: list, post, reply, help\r\n")?;
        stream.write_all(b"forum> ")?;
        
        // Simple command loop
        loop {
            let mut cmd_buffer = [0; 256];
            let bytes_read = stream.read(&mut cmd_buffer)?;
            if bytes_read == 0 { break; }
            
            let command = String::from_utf8_lossy(&cmd_buffer[..bytes_read]).trim();
            
            match command {
                "list" => {
                    stream.write_all(b"\r\nRecent threads:\r\n")?;
                    stream.write_all(b"[1] [SOLVED] pacman database lock\r\n")?;
                    stream.write_all(b"[2] [HELP] AUR package signing\r\n")?;
                    stream.write_all(b"[3] [DISCUSS] systemd vs openrc\r\n")?;
                }
                "help" => {
                    stream.write_all(b"\r\nCommands:\r\n")?;
                    stream.write_all(b"  list  - Show recent threads\r\n")?;
                    stream.write_all(b"  post  - Create new thread\r\n")?;
                    stream.write_all(b"  reply - Reply to thread\r\n")?;
                    stream.write_all(b"  help  - Show this help\r\n")?;
                }
                "quit" | "exit" => {
                    stream.write_all(b"Goodbye!\r\n")?;
                    break;
                }
                _ => {
                    stream.write_all(b"Unknown command. Type 'help'.\r\n")?;
                }
            }
            stream.write_all(b"forum> ")?;
        }
    } else {
        stream.write_all(b"Access denied: Arch Linux required\r\n")?;
    }
    
    Ok(())
}

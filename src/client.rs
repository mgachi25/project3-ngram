use crate::message::*;
use std::default::Default;
use std::io::Write;
use std::net::SocketAddr;
use std::net::{IpAddr,TcpStream};
use std::fs;

/// A client for interacting with the server at address `address`
pub struct Client {
    address: SocketAddr,
}
impl Default for Client {
    fn default() -> Self {
        Self::new("127.0.0.1", 7878)
    }
}

impl Client {
    // TODO:
    // Create a client that will connect to the server at `address` and `port`. You can create a
    // SocketAddr from an IpAddr and a port with `SocketAddr::new(addr, port)`.
    // You can create an IpAddr from a string with `address.parse().unwrap()`.
    pub fn new(address: &str, port: u16) -> Self {
        let ip_addr: IpAddr = address.parse().expect("Invalid IP address");
        let socket_addr = SocketAddr::new(ip_addr, port);
        Client { address: socket_addr }
    }

    // TODO:
    // This function is optional, but you may find it useful.
    // Convert the request to bytes, send it to the server, read the response to bytes, and convert
    // the response to a Response. If the response is invalid, return `None`.
    //
    // You can connect to the server with `std::net::TcpStream::connect(address)` function.
    // You can write to the stream with `stream.write_all(&bytes)`.
    // You can read from the stream by calling your `Response::from_bytes` function, since
    // `TcpStream` implements `Read`.
    fn send(&self, request: &Request) -> Option<Response> {
        // Convert the request to bytes
        let request_bytes = request.to_bytes();
        
        // Connect to the server
        let mut stream = TcpStream::connect(self.address).ok()?;
        stream.write_all(&request_bytes).ok()?;
        
        // Read the response from the server
        Response::from_bytes(&mut stream)
    }

    // TODO:
    // Read the file at `path` and send a `Publish` request to the server with its contents.
    // Return the response from the server.
    //
    // You can read the contents of a file with `let s = std::fs::read_to_string(path)`.
    pub fn publish_from_path(&self, path: &str) -> Option<Response> {
        // Read the file content
        let content = fs::read_to_string(path).ok()?;
        
        let request = Request::Publish { doc: content };
        self.send(&request)
    }
    // TODO:
    // Send a `Search` request to the server with the given `word`. Return the response from the
    // server.
    pub fn search(&self, word: &str) -> Option<Response> {
        let request = Request::Search { word: word.to_string() };
    
        // Send the request and return the response
        self.send(&request)
    }
    // TODO:
    // Send a `Retrieve` request to the server with the given `id`. Return the response from the
    // server.
    pub fn retrieve(&self, id: usize) -> Option<Response> {
        let request = Request::Retrieve { id };
        // Send the request and return the response
        self.send(&request)
    }
}

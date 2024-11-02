use crate::database::Database;
use crate::message::*;
use crate::pool::ThreadPool;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;


/// The number of workers in the server's thread pool
const WORKERS: usize = 16;

// TODO:
// Implement the `process_message` function. This function should take a `ServerState`, a `Request`,
// and a `TcpStream`. It should process the request and write the response to the stream.
// Processing the request should simply require calling the appropriate function on the database
// and then creating the appropriate response and turning it into bytes which are sent to along
// the stream by calling the `write_all` method.
fn process_message(state: Arc<ServerState>, request: Request, mut stream: TcpStream) {
    let response = match request {
        Request::Publish { doc } => {
            let id = state.database.publish(doc);
            Response::PublishSuccess(id)
        }
        Request::Search { word } => {
            let indices = state.database.search(&word);
            Response::SearchSuccess(indices)
        }
        Request::Retrieve { id } => {
            match state.database.retrieve(id) {
                Some(doc) => Response::RetrieveSuccess(doc),
                None => Response::Failure,
            }
        }
    };

    // Convert the response to bytes and send it over the stream
    let response_bytes = response.to_bytes();
    if stream.write_all(&response_bytes).is_err() {
        eprintln!("Failed to send response to client.");
    }
}

/// A struct that contains the state of the server
struct ServerState {
    /// The database that the server uses to store documents
    database: Database,
    /// The thread pool that the server uses to process requests
    pool: ThreadPool,
    /// A flag that indicates whether the server has been stopped
    is_stopped: AtomicBool,
}
impl ServerState {
    fn new() -> Self {
        Self {
            database: Database::new(),
            pool: ThreadPool::new(WORKERS),
            is_stopped: AtomicBool::new(false),
        }
    }
}

pub struct Server {
    state: Arc<ServerState>,
}
impl Server {
    // TODO:
    // Create a new server by using the `ServerState::new` function
    pub fn new() -> Self {
        Server {
            state: Arc::new(ServerState::new()),
        }
    }

    // TODO:
    // Spawn a thread that listens for incoming connections on the given port. When a connection is
    // established, add a task to the thread pool that deserializes the request, and processes it
    // using the `process_message` function.
    //
    // To listen for incoming connections, you can use the `std::net::TcpListener::bind` function.
    // To listen on the local address, you can call `TcpListener::bind(("127.0.0.1", port))`. The
    // resulting TcpListener can be used to accept incoming connections by calling the `accept`
    // method in a loop. This method blocks until a new connection is established, and then returns
    // a new TcpStream and the address of the remote peer. You should move this stream into the
    // task that you send to the thread pool.
    //
    // While looping to accept connections, you should also check the `is_stopped` flag in the
    // `ServerState` to see if the server has been stopped. If it has, you should break out of the
    // loop and return.
    fn listen(&self, port: u16) {
        let listener = TcpListener::bind(("127.0.0.1", port)).expect("Failed to bind to port");
        println!("Server listening on port {}", port);
        listener.set_nonblocking(true).expect("Cannot set non-blocking");

        for stream in listener.incoming() {
            // Check if the server has been stopped
            if self.state.is_stopped.load(Ordering::SeqCst) {
                return;
            }

            match stream {
                Ok(stream) => {
                    let state = Arc::clone(&self.state);
                    self.state.pool.execute(move || {
                        // Read and parse the request from the client
                        let request = Request::from_bytes(&stream);
                        if let Some(request) = request {
                            process_message(state, request, stream);
                        } else {
                            eprintln!("Failed to parse client request.");
                        }
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No connection available, sleep briefly before retrying
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    // This function has already been partially completed for you
    pub fn run(&self, port: u16) {
        // Set up a signal handler to stop the server when Ctrl-C is pressed
        let state = Arc::clone(&self.state);
        match ctrlc::try_set_handler(move || {
            println!("Stopping server...");
            state.is_stopped.store(true, Ordering::SeqCst);
        }) {
            Ok(_) => {}
            Err(ctrlc::Error::MultipleHandlers) => {}
            Err(e) => {
                panic!("Error setting Ctrl-C handler: {}", e);
            }
        }

        // TODO: Call the listen function and then loop (doing nothing) until the server has been stopped
        self.listen(port);
        while !self.state.is_stopped.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    pub fn stop(&self) {
        self.state.is_stopped.store(true, Ordering::SeqCst);
    }
}

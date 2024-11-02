use clap::{Parser, Subcommand};
use ngram::client::Client;
use ngram::server::Server;
use std::path::PathBuf;

// TODO:
// Fill out the `Args` struct to parse the command line arguments. You may find clap "subcommands"
// helpful.
/// An archive service allowing publishing and searching of books
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Start in either client or server mode
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand, Debug)]
enum Mode {
    /// Start the client to connect to the server
    Client {
        /// Server address to connect to
        server_address: String,
        /// Server port to connect to
        server_port: u16,
        /// Client action (publish, search, retrieve)
        #[command(subcommand)]
        action: ClientAction,
    },
    /// Start the server to listen for requests
    Server {
        /// Port to listen on
        listen_port: u16,
    },
}

#[derive(Subcommand, Debug)]
enum ClientAction {
    /// Publish a document to the archive
    Publish {
        /// Path to the document to publish
        document_path: PathBuf,
    },
    /// Search for a word in the archive
    Search {
        /// Word to search for
        word: String,
    },
    /// Retrieve a document by its ID
    Retrieve {
        /// Document ID to retrieve
        document_id: usize,
    },
}
// TODO:
// Inspect the contents of the `args` struct that has been created from the command line arguments
// the user passed. Depending on the arguments, either start a server or make a client and send the
// appropriate request. You may find it helpful to print the request response.
fn main() {
    let args = Args::parse();

    match args.mode {
        Mode::Client {
            server_address,
            server_port,
            action,
        } => {

            let client = Client::new(&server_address, server_port);

            match action {
                ClientAction::Publish { document_path } => {
                    println!("Publishing document: {:?}", document_path);
                    match client.publish_from_path(document_path.to_str().unwrap()) {
                        Some(response) => println!("Response: {}", response),
                        None => eprintln!("Failed to publish document"),
                    }
                }
                ClientAction::Search { word } => {
                    println!("Searching for word: {}", word);
                    match client.search(&word) {
                        Some(response) => println!("Response: {}", response),
                        None => eprintln!("Failed to search for word"),
                    }
                }
                ClientAction::Retrieve { document_id } => {
                    println!("Retrieving document with ID: {}", document_id);
                    match client.retrieve(document_id) {
                        Some(response) => println!("{}", response),
                        None => eprintln!("Failed to retrieve document"),
                    }
                }
            }
        }
        Mode::Server { listen_port } => {
            println!("Starting server on port: {}", listen_port);
            let server = Server::new();
            server.run(listen_port); // Placeholder for server start logic
        }
    }
}
use std::fmt;

/// A request from the client to the server
#[derive(Debug, PartialEq)]
pub enum Request {
    /// Add the document `doc` to the archive
    Publish { doc: String },
    /// Search for the word `word` in the archive
    Search { word: String },
    /// Retrieve the document with the index `id` from the archive
    Retrieve { id: usize },
}
impl Request {
    // TODO:
    // Convert the request `self` into a byte vector. See the assignment handout for suggestions on
    // how to represent the request as a series of bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        match self {
            Request::Publish { doc } => {
                bytes.push(0); // Type identifier for Publish
                let doc_bytes = doc.as_bytes();
                let doc_len = doc_bytes.len() as u64;
                bytes.extend(doc_len.to_be_bytes());
                bytes.extend(doc_bytes);
            }
            Request::Search { word } => {
                bytes.push(1); // Type identifier for Search
                let word_bytes = word.as_bytes();
                let word_len = word_bytes.len() as u64;
                bytes.extend(word_len.to_be_bytes());
                bytes.extend(word_bytes);
            }
            Request::Retrieve { id } => {
                bytes.push(2); // Type identifier for Retrieve
                bytes.extend(id.to_be_bytes());
            }
        }

        bytes
    }
    // TODO:
    // Read a request from `reader` and return it. Calling `to_bytes` from above and then calling
    // `from_bytes` should return the original request. If the request is invalid, return `None`.
    pub fn from_bytes<R: std::io::Read>(mut reader: R) -> Option<Self> {
        let mut type_byte = [0; 1];
        reader.read_exact(&mut type_byte).ok()?;

        match type_byte[0] {
            0 => {
                // Publish request
                let mut len_bytes = [0; 8];
                reader.read_exact(&mut len_bytes).ok()?;
                let doc_len = u64::from_be_bytes(len_bytes) as usize;
                let mut doc_bytes = vec![0; doc_len];
                reader.read_exact(&mut doc_bytes).ok()?;
                let doc = String::from_utf8(doc_bytes).ok()?;
                Some(Request::Publish { doc })
            }
            1 => {
                // Search request
                let mut len_bytes = [0; 8];
                reader.read_exact(&mut len_bytes).ok()?;
                let word_len = u64::from_be_bytes(len_bytes) as usize;
                let mut word_bytes = vec![0; word_len];
                reader.read_exact(&mut word_bytes).ok()?;
                let word = String::from_utf8(word_bytes).ok()?;
                Some(Request::Search { word })
            }
            2 => {
                // Retrieve request
                let mut id_bytes = [0; 8];
                reader.read_exact(&mut id_bytes).ok()?;
                let id = usize::from_be_bytes(id_bytes);
                Some(Request::Retrieve { id })
            }
            _ => None, // Unknown type, return None
        }
    }
}

/// A response from the server to the client
#[derive(Debug, PartialEq)]
pub enum Response {
    /// The document was successfully added to the archive with the given index
    PublishSuccess(usize),
    /// The search for the word was successful, and the indices of the documents containing the
    /// word are returned
    SearchSuccess(Vec<usize>),
    /// The retrieval of the document was successful, and the document is returned
    RetrieveSuccess(String),
    /// The request failed
    Failure,
}
impl Response {
    // TODO:
    // Convert the request `self` into a byte vector. See the assignment handout for suggestions on
    // how to represent the request as a series of bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        match self {
            Response::PublishSuccess(index) => {
                bytes.push(0); // Type identifier for PublishSuccess
                bytes.extend(index.to_be_bytes()); 
            }
            Response::SearchSuccess(indices) => {
                bytes.push(1); // Type identifier for SearchSuccess
                let len = indices.len() as u64;
                bytes.extend(len.to_be_bytes()); 
                for index in indices {
                    bytes.extend(index.to_be_bytes()); 
                }
            }
            Response::RetrieveSuccess(document) => {
                bytes.push(2); // Type identifier for RetrieveSuccess
                let doc_bytes = document.as_bytes();
                let doc_len = doc_bytes.len() as u64;
                bytes.extend(doc_len.to_be_bytes());
                bytes.extend(doc_bytes);
            }
            Response::Failure => {
                bytes.push(3); // Type identifier for Failure
            }
        }

        bytes
    }
    // TODO:
    // Read a request from `reader` and return it. Calling `to_bytes` from above and then calling
    // `from_bytes` should return the original request. If the request is invalid, return `None`.
    pub fn from_bytes<R: std::io::Read>(mut reader: R) -> Option<Self> {
        let mut type_byte = [0; 1];
        reader.read_exact(&mut type_byte).ok()?;

        match type_byte[0] {
            0 => {
                // PublishSuccess response
                let mut index_bytes = [0; 8];
                reader.read_exact(&mut index_bytes).ok()?;
                let index = usize::from_be_bytes(index_bytes);
                Some(Response::PublishSuccess(index))
            }
            1 => {
                // SearchSuccess response
                let mut len_bytes = [0; 8];
                reader.read_exact(&mut len_bytes).ok()?;
                let len = u64::from_be_bytes(len_bytes) as usize;
                let mut indices = Vec::with_capacity(len);
                for _ in 0..len {
                    let mut index_bytes = [0; 8];
                    reader.read_exact(&mut index_bytes).ok()?;
                    indices.push(usize::from_be_bytes(index_bytes));
                }
                Some(Response::SearchSuccess(indices))
            }
            2 => {
                // RetrieveSuccess response
                let mut len_bytes = [0; 8];
                reader.read_exact(&mut len_bytes).ok()?;
                let doc_len = u64::from_be_bytes(len_bytes) as usize;
                let mut doc_bytes = vec![0; doc_len];
                reader.read_exact(&mut doc_bytes).ok()?;
                let document = String::from_utf8(doc_bytes).ok()?;
                Some(Response::RetrieveSuccess(document))
            }
            3 => {
                // Failure response
                Some(Response::Failure)
            }
            _ => None, 
        }
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::PublishSuccess(id) => {
                write!(f, "PublishSuccess(Document ID: {})", id)
            }
            Response::SearchSuccess(indices) => {
                write!(f, "SearchSuccess(Document IDs: {:?})", indices)
            }
            Response::RetrieveSuccess(content) => {
                // Limit the output to the first 100 characters for readability
                let preview = if content.len() > 100 {
                    format!("{}...", &content[..100])
                } else {
                    content.clone()
                };
                write!(f, "RetrieveSuccess(\"{}\")", preview)
            }
            Response::Failure => write!(f, "Response: Failure"),
        }
    }
}
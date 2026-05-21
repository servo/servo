/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Copyright 2026      The Servo Developers
// Copyright 2016-2025 mime-multipart Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//
// This file originates from github.com/mikedilger/mime-multipart and https://github.com/gw-de/mime-multipart-hyper1.
// The file as is, is licensed under MPL-2.0. Any code that is originally from mime-multipart
// or its fork mime-multipart-hyper1 are additionally licensed under Apache-2.0 and MIT, as
// per the original license.

mod error {
    use std::error::Error as StdError;
    use std::fmt::{self, Display};
    use std::io;

    use http::header::ToStrError;

    /// An error type for the `mime-multipart` crate.
    pub enum Error {
        /// The Hyper request did not have a Content-Type header.
        NoRequestContentType,
        /// The Hyper request Content-Type top-level Mime was not `Multipart`.
        NotMultipart,
        /// The Content-Type header failed to specify boundary token.
        BoundaryNotSpecified,
        /// A multipart section contained only partial headers.
        PartialHeaders,
        EofBeforeFirstBoundary,
        NoCrLfAfterBoundary,
        EofInPartHeaders,
        EofInFile,
        EofInPart,
        InvalidHeaderNameOrValue,
        HeaderValueNotMime,
        ToStr(ToStrError),
        /// An HTTP parsing error from a multipart section.
        Httparse(httparse::Error),
        /// An I/O error.
        Io(io::Error),
    }

    impl From<io::Error> for Error {
        fn from(err: io::Error) -> Error {
            Error::Io(err)
        }
    }

    impl From<httparse::Error> for Error {
        fn from(err: httparse::Error) -> Error {
            Error::Httparse(err)
        }
    }

    impl Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Error::Httparse(ref e) => format!("Httparse: {:?}", e).fmt(f),
                Error::Io(ref e) => format!("Io: {}", e).fmt(f),
                Error::ToStr(ref e) => format!("ToStr: {}", e).fmt(f),
                Error::NoRequestContentType => "NoRequestContentType".to_string().fmt(f),
                Error::NotMultipart => "NotMultipart".to_string().fmt(f),
                Error::BoundaryNotSpecified => "BoundaryNotSpecified".to_string().fmt(f),
                Error::PartialHeaders => "PartialHeaders".to_string().fmt(f),
                Error::EofBeforeFirstBoundary => "EofBeforeFirstBoundary".to_string().fmt(f),
                Error::NoCrLfAfterBoundary => "NoCrLfAfterBoundary".to_string().fmt(f),
                Error::EofInPartHeaders => "EofInPartHeaders".to_string().fmt(f),
                Error::EofInFile => "EofInFile".to_string().fmt(f),
                Error::EofInPart => "EofInPart".to_string().fmt(f),
                Error::InvalidHeaderNameOrValue => "InvalidHeaderNameOrValue".to_string().fmt(f),
                Error::HeaderValueNotMime => "HeaderValueNotMime".to_string().fmt(f),
            }
        }
    }

    impl fmt::Debug for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self)?;
            if self.source().is_some() {
                write!(f, ": {:?}", self.source().unwrap())?; // recurse
            }
            Ok(())
        }
    }

    impl StdError for Error {
        fn description(&self) -> &str {
            match *self {
                Error::NoRequestContentType => {
                    "The Hyper request did not have a Content-Type header."
                },
                Error::NotMultipart => {
                    "The Hyper request Content-Type top-level Mime was not multipart."
                },
                Error::BoundaryNotSpecified => {
                    "The Content-Type header failed to specify a boundary token."
                },
                Error::PartialHeaders => "A multipart section contained only partial headers.",
                Error::EofBeforeFirstBoundary => {
                    "The request body ended prior to reaching the expected starting boundary."
                },
                Error::NoCrLfAfterBoundary => "Missing CRLF after boundary.",
                Error::EofInPartHeaders => {
                    "The request body ended prematurely while parsing headers of a multipart part."
                },
                Error::EofInFile => {
                    "The request body ended prematurely while streaming a file part."
                },
                Error::EofInPart => {
                    "The request body ended prematurely while reading a multipart part."
                },
                Error::Httparse(_) => {
                    "A parse error occurred while parsing the headers of a multipart section."
                },
                Error::Io(_) => "An I/O error occurred.",
                Error::InvalidHeaderNameOrValue => "Parsing to HeaderName or HeaderValue failed",
                Error::HeaderValueNotMime => "HeaderValue could not be parsed to Mime",
                Error::ToStr(_) => "A ToStr error occurred.",
            }
        }
    }
}

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::ops::Drop;
use std::path::PathBuf;
use std::str::FromStr;

use buf_read_ext::BufReadExt;
pub use error::Error;
use http::header::{HeaderMap, HeaderName, HeaderValue};
use mime::Mime;

/// A multipart part which is not a file (stored in memory)
#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

/// A file that is to be inserted into a `multipart/*` or alternatively an uploaded file that
/// was received as part of `multipart/*` parsing.
#[derive(Debug, PartialEq)]
pub struct FilePart {
    /// The headers of the part
    pub headers: HeaderMap,
    /// A temporary file containing the file content
    pub path: PathBuf,
    /// Optionally, the size of the file.  This is filled when multiparts are parsed, but is
    /// not necessary when they are generated.
    pub size: Option<usize>,
    // The temporary directory the upload was put into, saved for the Drop trait
    tempdir: Option<PathBuf>,
}
impl FilePart {
    /// Create a new temporary FilePart (when created this way, the file will be
    /// deleted once the FilePart object goes out of scope).
    pub fn create(headers: HeaderMap) -> Result<FilePart, Error> {
        // TODO: Do we really need a dir with only one file in it?
        // Perhaps we just just do a tempfile, then we also have
        // one cleanup step less!
        // Setup a file to capture the contents.
        let mut path = tempfile::Builder::new()
            .prefix("mime_multipart")
            .tempdir()?
            .keep();
        let tempdir = Some(path.clone());
        // The directory name is already guaranteed to be unique.
        path.push("part");
        Ok(FilePart {
            headers,
            path,
            size: None,
            tempdir,
        })
    }
}
impl Drop for FilePart {
    fn drop(&mut self) {
        if let Some(tempdir) = &self.tempdir {
            let _ = std::fs::remove_file(&self.path);
            let _ = std::fs::remove_dir(tempdir);
        }
    }
}

/// A multipart part which could be either a file, in memory, or another multipart
/// container containing nested parts.
#[derive(Debug)]
pub enum Node {
    /// A part in memory
    Part(Part),
    /// A part streamed to a file
    File(FilePart),
    /// A container of nested multipart parts
    Multipart((HeaderMap, Vec<Node>)),
}

/// Parse a MIME `multipart/*` from a `Read`able stream into a `Vec` of `Node`s, streaming
/// files to disk and keeping the rest in memory.  Recursive `multipart/*` parts will are
/// parsed as well and returned within a `Node::Multipart` variant.
///
/// If `always_use_files` is true, all parts will be streamed to files.  If false, only parts
/// with a `ContentDisposition` header set to `Attachment` or otherwise containing a `Filename`
/// parameter will be streamed to files.
///
/// It is presumed that you have the `Headers` already and the stream starts at the body.
/// If the headers are still in the stream, use `read_multipart()` instead.
pub fn read_multipart_body<S: Read>(
    stream: &mut S,
    headers: &HeaderMap,
    always_use_files: bool,
) -> Result<Vec<Node>, Error> {
    let mut reader = BufReader::with_capacity(4096, stream);
    inner(&mut reader, headers, always_use_files)
}

fn inner<R: BufRead>(
    reader: &mut R,
    headers: &HeaderMap,
    always_use_files: bool,
) -> Result<Vec<Node>, Error> {
    let mut nodes: Vec<Node> = Vec::new();
    let mut buf: Vec<u8> = Vec::new();

    let boundary = get_multipart_boundary(headers)?;

    // Read past the initial boundary
    let (_, found) = reader.stream_until_token(&boundary, &mut buf)?;
    if !found {
        return Err(Error::EofBeforeFirstBoundary);
    }

    // Define the boundary, including the line terminator preceding it.
    // Use their first line terminator to determine whether to use CRLF or LF.
    let (lt, ltlt, lt_boundary) = {
        let peeker = reader.fill_buf()?;
        if peeker.len() > 1 && &peeker[..2] == b"\r\n" {
            let mut output = Vec::with_capacity(2 + boundary.len());
            output.push(b'\r');
            output.push(b'\n');
            output.extend(boundary.clone());
            (vec![b'\r', b'\n'], vec![b'\r', b'\n', b'\r', b'\n'], output)
        } else if !peeker.is_empty() && peeker[0] == b'\n' {
            let mut output = Vec::with_capacity(1 + boundary.len());
            output.push(b'\n');
            output.extend(boundary.clone());
            (vec![b'\n'], vec![b'\n', b'\n'], output)
        } else {
            return Err(Error::NoCrLfAfterBoundary);
        }
    };

    loop {
        // If the next two lookahead characters are '--', parsing is finished.
        {
            let peeker = reader.fill_buf()?;
            if peeker.len() >= 2 && &peeker[..2] == b"--" {
                return Ok(nodes);
            }
        }

        // Read the line terminator after the boundary
        let (_, found) = reader.stream_until_token(&lt, &mut buf)?;
        if !found {
            return Err(Error::NoCrLfAfterBoundary);
        }

        // Read the headers (which end in 2 line terminators)
        buf.truncate(0); // start fresh
        let (_, found) = reader.stream_until_token(&ltlt, &mut buf)?;
        if !found {
            return Err(Error::EofInPartHeaders);
        }

        // Keep the 2 line terminators as httparse will expect it
        buf.extend(ltlt.iter().cloned());

        // Parse the headers
        let part_headers = {
            let mut header_memory = [httparse::EMPTY_HEADER; 4];
            match httparse::parse_headers(&buf, &mut header_memory) {
                Ok(httparse::Status::Complete((_, raw_headers))) => {
                    let mut headers = HeaderMap::new();
                    for header in raw_headers {
                        if header.value.is_empty() {
                            break;
                        }
                        let trim = header
                            .value
                            .iter()
                            .rev()
                            .take_while(|&&x| x == b' ')
                            .count();
                        let value = &header.value[..header.value.len() - trim];

                        let header_value = match HeaderValue::from_bytes(value) {
                            Ok(value) => value,
                            Err(_) => return Err(Error::InvalidHeaderNameOrValue),
                        };

                        let header_name = header.name.to_owned();
                        let header_name = match HeaderName::from_str(&header_name) {
                            Ok(value) => value,
                            Err(_) => return Err(Error::InvalidHeaderNameOrValue),
                        };
                        headers.append(header_name, header_value);
                    }
                    Ok(headers)
                },
                Ok(httparse::Status::Partial) => Err(Error::PartialHeaders),
                Err(err) => Err(From::from(err)),
            }?
        };

        // Check for a nested multipart
        let nested = {
            match part_headers.get("content-type") {
                Some(ct) => match ct.to_str() {
                    Ok(value) => match Mime::from_str(value) {
                        Ok(mime) => mime.type_() == mime::MULTIPART,
                        Err(_) => return Err(Error::HeaderValueNotMime),
                    },
                    Err(err) => return Err(Error::ToStr(err)),
                },
                None => false,
            }
        };
        if nested {
            // Recurse:
            let inner_nodes = inner(reader, &part_headers, always_use_files)?;
            nodes.push(Node::Multipart((part_headers, inner_nodes)));
            continue;
        }

        let is_file = always_use_files || {
            match part_headers.get("content-disposition") {
                Some(content) => match content.to_str() {
                    Ok(value) => value.contains("attachment") || value.contains("filename"),
                    Err(err) => return Err(Error::ToStr(err)),
                },
                None => false,
            }
        };
        if is_file {
            // Setup a file to capture the contents.
            let mut filepart = FilePart::create(part_headers)?;
            let mut file = File::create(filepart.path.clone())?;

            // Stream out the file.
            let (read, found) = reader.stream_until_token(&lt_boundary, &mut file)?;
            if !found {
                return Err(Error::EofInFile);
            }
            filepart.size = Some(read);

            // TODO: Handle Content-Transfer-Encoding.  RFC 7578 section 4.7 deprecated
            // this, and the authors state "Currently, no deployed implementations that
            // send such bodies have been discovered", so this is very low priority.

            nodes.push(Node::File(filepart));
        } else {
            buf.truncate(0); // start fresh
            let (_, found) = reader.stream_until_token(&lt_boundary, &mut buf)?;
            if !found {
                return Err(Error::EofInPart);
            }

            nodes.push(Node::Part(Part {
                headers: part_headers,
                body: buf.clone(),
            }));
        }
    }
}

/// Get the `multipart/*` boundary string from `hyper::Headers`
pub fn get_multipart_boundary(headers: &HeaderMap) -> Result<Vec<u8>, Error> {
    // Verify that the request is 'Content-Type: multipart/*'.
    let mime = match headers.get("content-type") {
        Some(ct) => match ct.to_str() {
            Ok(value) => match Mime::from_str(value) {
                Ok(value) => value,
                Err(_) => return Err(Error::HeaderValueNotMime),
            },
            Err(err) => return Err(Error::ToStr(err)),
        },
        None => return Err(Error::NoRequestContentType),
    };
    let top_level = mime.type_();

    if top_level != mime::MULTIPART {
        return Err(Error::NotMultipart);
    }

    match mime.get_param(mime::BOUNDARY) {
        None => Err(Error::BoundaryNotSpecified),
        Some(content) => {
            let mut boundary = vec![];
            boundary.extend(b"--".iter().cloned());
            boundary.extend(content.to_string().as_bytes());
            Ok(boundary)
        },
    }
}

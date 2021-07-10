/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Adapted from an implementation in reqwest.

/*!
A potentially non-blocking response decoder.

The decoder wraps a stream of chunks and produces a new stream of decompressed chunks.
The decompressed chunks aren't guaranteed to align to the compressed ones.

If the response is plaintext then no additional work is carried out.
Chunks are just passed along.

If the response is gzip, then the chunks are decompressed into a buffer.
Slices of that buffer are emitted as new chunks.

This module consists of a few main types:

- `ReadableChunks` is a `Read`-like wrapper around a stream
- `Decoder` is a layer over `ReadableChunks` that applies the right decompression

The following types directly support the gzip compression case:

- `Pending` is a non-blocking constructor for a `Decoder` in case the body needs to be checked for EOF
- `Peeked` is a buffer that keeps a few bytes available so `libflate`s `read_exact` calls won't fail
*/

use crate::connector::BUF_SIZE;
use brotli::Decompressor;
use bytes::{Buf, BufMut, BytesMut};
use flate2::read::DeflateDecoder;
use futures::{Async, Future, Poll, Stream};
use hyper::header::{HeaderValue, CONTENT_ENCODING, TRANSFER_ENCODING};
use hyper::{self, Body, Chunk, Response};
use libflate::non_blocking::gzip;
use std::cmp;
use std::fmt;
use std::io::{self, Read};
use std::mem;

pub enum Error {
    Io(io::Error),
    Hyper(hyper::error::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<hyper::error::Error> for Error {
    fn from(err: hyper::error::Error) -> Error {
        Error::Hyper(err)
    }
}

const INIT_BUFFER_SIZE: usize = 8192;

/// A response decompressor over a non-blocking stream of chunks.
///
/// The inner decoder may be constructed asynchronously.
pub struct Decoder {
    inner: Inner,
}

#[derive(PartialEq)]
enum DecoderType {
    Gzip,
    Brotli,
    Deflate,
}

enum Inner {
    /// A `PlainText` decoder just returns the response content as is.
    PlainText(Body),
    /// A `Gzip` decoder will uncompress the gzipped response content before returning it.
    Gzip(Gzip),
    /// A `Delfate` decoder will uncompress the inflated response content before returning it.
    Deflate(Deflate),
    /// A `Brotli` decoder will uncompress the brotli-encoded response content before returning it.
    Brotli(Brotli),
    /// A decoder that doesn't have a value yet.
    Pending(Pending),
}

/// A future attempt to poll the response body for EOF so we know whether to use gzip or not.
struct Pending {
    body: ReadableChunks<Body>,
    type_: DecoderType,
}

/// A gzip decoder that reads from a `libflate::gzip::Decoder` into a `BytesMut` and emits the results
/// as a `Chunk`.
struct Gzip {
    inner: Box<gzip::Decoder<Peeked<ReadableChunks<Body>>>>,
    buf: BytesMut,
}

impl fmt::Debug for Decoder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Decoder").finish()
    }
}

impl Decoder {
    /// A plain text decoder.
    ///
    /// This decoder will emit the underlying chunks as-is.
    #[inline]
    fn plain_text(body: Body) -> Decoder {
        Decoder {
            inner: Inner::PlainText(body),
        }
    }

    /// A pending decoder.
    ///
    /// This decoder will buffer and decompress chunks that are encoded in the expected format.
    #[inline]
    fn pending(body: Body, type_: DecoderType) -> Decoder {
        Decoder {
            inner: Inner::Pending(Pending {
                body: ReadableChunks::new(body),
                type_: type_,
            }),
        }
    }

    /// Constructs a Decoder from a hyper request.
    ///
    /// A decoder is just a wrapper around the hyper request that knows
    /// how to decode the content body of the request.
    ///
    /// Uses the correct variant by inspecting the Content-Encoding header.
    pub fn detect(response: Response<Body>) -> Response<Decoder> {
        let values = response
            .headers()
            .get_all(CONTENT_ENCODING)
            .iter()
            .chain(response.headers().get_all(TRANSFER_ENCODING).iter());
        let decoder = values.fold(None, |acc, enc| {
            acc.or_else(|| {
                if enc == HeaderValue::from_static("gzip") {
                    Some(DecoderType::Gzip)
                } else if enc == HeaderValue::from_static("br") {
                    Some(DecoderType::Brotli)
                } else if enc == HeaderValue::from_static("deflate") {
                    Some(DecoderType::Deflate)
                } else {
                    None
                }
            })
        });
        match decoder {
            Some(type_) => response.map(|r| Decoder::pending(r, type_)),
            None => response.map(Decoder::plain_text),
        }
    }
}

impl Stream for Decoder {
    type Item = Chunk;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // Do a read or poll for a pending decoder value.
        let new_value = match self.inner {
            Inner::Pending(ref mut future) => match future.poll() {
                Ok(Async::Ready(inner)) => inner,
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(e) => return Err(e.into()),
            },
            Inner::PlainText(ref mut body) => return body.poll().map_err(|e| e.into()),
            Inner::Gzip(ref mut decoder) => return decoder.poll(),
            Inner::Brotli(ref mut decoder) => return decoder.poll(),
            Inner::Deflate(ref mut decoder) => return decoder.poll(),
        };

        self.inner = new_value;
        self.poll()
    }
}

impl Future for Pending {
    type Item = Inner;
    type Error = hyper::error::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let body_state = match self.body.poll_stream() {
            Ok(Async::Ready(state)) => state,
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Err(e) => return Err(e),
        };

        let body = mem::replace(&mut self.body, ReadableChunks::new(Body::empty()));
        // libflate does a read_exact([0; 2]), so its impossible to tell
        // if the stream was empty, or truly had an UnexpectedEof.
        // Therefore, we need to check for EOF first.
        match body_state {
            StreamState::Eof => Ok(Async::Ready(Inner::PlainText(Body::empty()))),
            StreamState::HasMore => Ok(Async::Ready(match self.type_ {
                DecoderType::Gzip => Inner::Gzip(Gzip::new(body)),
                DecoderType::Brotli => Inner::Brotli(Brotli::new(body)),
                DecoderType::Deflate => Inner::Deflate(Deflate::new(body)),
            })),
        }
    }
}

impl Gzip {
    fn new(stream: ReadableChunks<Body>) -> Self {
        Gzip {
            buf: BytesMut::with_capacity(INIT_BUFFER_SIZE),
            inner: Box::new(gzip::Decoder::new(Peeked::new(stream))),
        }
    }
}

#[allow(unsafe_code)]
fn poll_with_read(reader: &mut dyn Read, buf: &mut BytesMut) -> Poll<Option<Chunk>, Error> {
    if buf.remaining_mut() == 0 {
        buf.reserve(INIT_BUFFER_SIZE);
    }

    // The buffer contains uninitialised memory so getting a readable slice is unsafe.
    // We trust the reader not to read from the memory given.
    //
    // To be safe, this memory could be zeroed before passing to the reader.
    // Otherwise we might need to deal with the case where the reader panics.
    let read = {
        let mut buf = unsafe { buf.bytes_mut() };
        reader.read(&mut buf)
    };

    match read {
        Ok(read) if read == 0 => Ok(Async::Ready(None)),
        Ok(read) => {
            unsafe { buf.advance_mut(read) };
            let chunk = Chunk::from(buf.split_to(read).freeze());

            Ok(Async::Ready(Some(chunk)))
        },
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(Async::NotReady),
        Err(e) => Err(e.into()),
    }
}

impl Stream for Gzip {
    type Item = Chunk;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        poll_with_read(&mut self.inner, &mut self.buf)
    }
}

/// A brotli decoder that reads from a `brotli::Decompressor` into a `BytesMut` and emits the results
/// as a `Chunk`.
struct Brotli {
    inner: Box<Decompressor<Peeked<ReadableChunks<Body>>>>,
    buf: BytesMut,
}

impl Brotli {
    fn new(stream: ReadableChunks<Body>) -> Self {
        Self {
            buf: BytesMut::with_capacity(INIT_BUFFER_SIZE),
            inner: Box::new(Decompressor::new(Peeked::new(stream), BUF_SIZE)),
        }
    }
}

impl Stream for Brotli {
    type Item = Chunk;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        poll_with_read(&mut self.inner, &mut self.buf)
    }
}

/// A deflate decoder that reads from a `deflate::Decoder` into a `BytesMut` and emits the results
/// as a `Chunk`.
struct Deflate {
    inner: Box<DeflateDecoder<Peeked<ReadableChunks<Body>>>>,
    buf: BytesMut,
}

impl Deflate {
    fn new(stream: ReadableChunks<Body>) -> Self {
        Self {
            buf: BytesMut::with_capacity(INIT_BUFFER_SIZE),
            inner: Box::new(DeflateDecoder::new(Peeked::new(stream))),
        }
    }
}

impl Stream for Deflate {
    type Item = Chunk;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        poll_with_read(&mut self.inner, &mut self.buf)
    }
}

/// A `Read`able wrapper over a stream of chunks.
pub struct ReadableChunks<S> {
    state: ReadState,
    stream: S,
}

enum ReadState {
    /// A chunk is ready to be read from.
    Ready(Chunk),
    /// The next chunk isn't ready yet.
    NotReady,
    /// The stream has finished.
    Eof,
}

enum StreamState {
    /// More bytes can be read from the stream.
    HasMore,
    /// No more bytes can be read from the stream.
    Eof,
}

/// A buffering reader that ensures `Read`s return at least a few bytes.
struct Peeked<R> {
    state: PeekedState,
    peeked_buf: [u8; 10],
    pos: usize,
    inner: R,
}

enum PeekedState {
    /// The internal buffer hasn't filled yet.
    NotReady,
    /// The internal buffer can be read.
    Ready(usize),
}

impl<R> Peeked<R> {
    #[inline]
    fn new(inner: R) -> Self {
        Peeked {
            state: PeekedState::NotReady,
            peeked_buf: [0; 10],
            inner: inner,
            pos: 0,
        }
    }

    #[inline]
    fn ready(&mut self) {
        self.state = PeekedState::Ready(self.pos);
        self.pos = 0;
    }

    #[inline]
    fn not_ready(&mut self) {
        self.state = PeekedState::NotReady;
        self.pos = 0;
    }
}

impl<R: Read> Read for Peeked<R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            match self.state {
                PeekedState::Ready(peeked_buf_len) => {
                    let len = cmp::min(buf.len(), peeked_buf_len - self.pos);
                    let start = self.pos;
                    let end = self.pos + len;

                    buf[..len].copy_from_slice(&self.peeked_buf[start..end]);
                    self.pos += len;
                    if self.pos == peeked_buf_len {
                        self.not_ready();
                    }

                    return Ok(len);
                },
                PeekedState::NotReady => {
                    let read = self.inner.read(&mut self.peeked_buf[self.pos..]);

                    match read {
                        Ok(0) => self.ready(),
                        Ok(read) => {
                            self.pos += read;
                            if self.pos == self.peeked_buf.len() {
                                self.ready();
                            }
                        },
                        Err(e) => return Err(e),
                    }
                },
            };
        }
    }
}

impl<S> ReadableChunks<S> {
    #[inline]
    fn new(stream: S) -> Self {
        ReadableChunks {
            state: ReadState::NotReady,
            stream: stream,
        }
    }
}

impl<S> fmt::Debug for ReadableChunks<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ReadableChunks").finish()
    }
}

impl<S> Read for ReadableChunks<S>
where
    S: Stream<Item = Chunk, Error = hyper::error::Error>,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let ret;
            match self.state {
                ReadState::Ready(ref mut chunk) => {
                    let len = cmp::min(buf.len(), chunk.remaining());

                    buf[..len].copy_from_slice(&chunk[..len]);
                    chunk.advance(len);
                    if chunk.is_empty() {
                        ret = len;
                    } else {
                        return Ok(len);
                    }
                },
                ReadState::NotReady => match self.poll_stream() {
                    Ok(Async::Ready(StreamState::HasMore)) => continue,
                    Ok(Async::Ready(StreamState::Eof)) => return Ok(0),
                    Ok(Async::NotReady) => return Err(io::ErrorKind::WouldBlock.into()),
                    Err(e) => {
                        return Err(io::Error::new(io::ErrorKind::Other, e));
                    },
                },
                ReadState::Eof => return Ok(0),
            }
            self.state = ReadState::NotReady;
            return Ok(ret);
        }
    }
}

impl<S> ReadableChunks<S>
where
    S: Stream<Item = Chunk, Error = hyper::error::Error>,
{
    /// Poll the readiness of the inner reader.
    ///
    /// This function will update the internal state and return a simplified
    /// version of the `ReadState`.
    fn poll_stream(&mut self) -> Poll<StreamState, hyper::error::Error> {
        match self.stream.poll() {
            Ok(Async::Ready(Some(chunk))) => {
                self.state = ReadState::Ready(chunk);

                Ok(Async::Ready(StreamState::HasMore))
            },
            Ok(Async::Ready(None)) => {
                self.state = ReadState::Eof;

                Ok(Async::Ready(StreamState::Eof))
            },
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(e),
        }
    }
}

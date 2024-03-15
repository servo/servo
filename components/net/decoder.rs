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

use std::io::{self, Read};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use std::{cmp, fmt, mem};

use brotli::Decompressor;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use flate2::read::DeflateDecoder;
use futures::task::{Context, Poll};
use futures::{Future, Stream};
use hyper::header::{HeaderValue, CONTENT_ENCODING, TRANSFER_ENCODING};
use hyper::{self, Body, Response};
use libflate::non_blocking::gzip;

use crate::connector::BUF_SIZE;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Hyper(hyper::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Error {
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
/// as a `Bytes`.
struct Gzip {
    inner: Box<gzip::Decoder<Peeked<ReadableChunks<Body>>>>,
    buf: BytesMut,
    reader: Arc<Mutex<ReadableChunks<Body>>>,
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
                type_,
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
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Do a read or poll for a pending decoder value.
        let new_value = match self.inner {
            Inner::Pending(ref mut future) => match Pin::new(future).poll(cx) {
                Poll::Ready(inner) => inner,
                Poll::Pending => return Poll::Pending,
            },
            Inner::PlainText(ref mut body) => {
                return Pin::new(body).poll_next(cx).map_err(|e| e.into())
            },
            Inner::Gzip(ref mut decoder) => return Pin::new(decoder).poll_next(cx),
            Inner::Brotli(ref mut decoder) => return Pin::new(decoder).poll_next(cx),
            Inner::Deflate(ref mut decoder) => return Pin::new(decoder).poll_next(cx),
        };

        //
        self.inner = new_value;
        self.poll_next(cx)
    }
}

impl Future for Pending {
    type Output = Inner;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let body_state = match self.body.poll_stream(cx) {
            Poll::Ready(state) => state,
            Poll::Pending => return Poll::Pending,
        };

        let body = mem::replace(&mut self.body, ReadableChunks::new(Body::empty()));
        // libflate does a read_exact([0; 2]), so its impossible to tell
        // if the stream was empty, or truly had an UnexpectedEof.
        // Therefore, we need to check for EOF first.
        match body_state {
            StreamState::Eof => Poll::Ready(Inner::PlainText(Body::empty())),
            StreamState::HasMore => Poll::Ready(match self.type_ {
                DecoderType::Gzip => Inner::Gzip(Gzip::new(body)),
                DecoderType::Brotli => Inner::Brotli(Brotli::new(body)),
                DecoderType::Deflate => Inner::Deflate(Deflate::new(body)),
            }),
        }
    }
}

impl Gzip {
    fn new(stream: ReadableChunks<Body>) -> Self {
        let stream = Arc::new(Mutex::new(stream));
        let reader = stream.clone();
        Gzip {
            buf: BytesMut::with_capacity(INIT_BUFFER_SIZE),
            inner: Box::new(gzip::Decoder::new(Peeked::new(stream))),
            reader,
        }
    }
}

#[allow(unsafe_code)]
fn poll_with_read(reader: &mut dyn Read, buf: &mut BytesMut) -> Poll<Option<Result<Bytes, Error>>> {
    // Ensure a full size buffer is available.
    // `reserve` is optimized to reclaim space over allocating.
    buf.reserve(INIT_BUFFER_SIZE);

    // The buffer contains uninitialised memory so getting a readable slice is unsafe.
    // We trust the reader not to read from the memory given.
    //
    // To be safe, this memory could be zeroed before passing to the reader.
    // Otherwise we might need to deal with the case where the reader panics.

    let read = {
        let buf = unsafe {
            let ptr = buf.chunk_mut().as_mut_ptr();
            std::slice::from_raw_parts_mut(ptr, buf.capacity())
        };
        reader.read(&mut *buf)
    };

    match read {
        Ok(0) => Poll::Ready(None),
        Ok(read) => {
            unsafe { buf.advance_mut(read) };
            let chunk = buf.split_to(read).freeze();
            Poll::Ready(Some(Ok(chunk)))
        },
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Poll::Pending,
        Err(e) => Poll::Ready(Some(Err(e.into()))),
    }
}

impl Stream for Gzip {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = self.buf.clone();
        if let Ok(mut reader) = self.reader.lock() {
            reader.waker = Some(cx.waker().clone());
        }
        poll_with_read(&mut self.inner, &mut buf)
    }
}

/// A brotli decoder that reads from a `brotli::Decompressor` into a `BytesMut` and emits the results
/// as a `Bytes`.
struct Brotli {
    inner: Box<Decompressor<Peeked<ReadableChunks<Body>>>>,
    buf: BytesMut,
    reader: Arc<Mutex<ReadableChunks<Body>>>,
}

impl Brotli {
    fn new(stream: ReadableChunks<Body>) -> Self {
        let stream = Arc::new(Mutex::new(stream));
        let reader = stream.clone();
        Self {
            buf: BytesMut::with_capacity(INIT_BUFFER_SIZE),
            inner: Box::new(Decompressor::new(Peeked::new(stream), BUF_SIZE)),
            reader,
        }
    }
}

impl Stream for Brotli {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = self.buf.clone();
        if let Ok(mut reader) = self.reader.lock() {
            reader.waker = Some(cx.waker().clone());
        }
        poll_with_read(&mut self.inner, &mut buf)
    }
}

/// A deflate decoder that reads from a `deflate::Decoder` into a `BytesMut` and emits the results
/// as a `Bytes`.
struct Deflate {
    inner: Box<DeflateDecoder<Peeked<ReadableChunks<Body>>>>,
    buf: BytesMut,
    reader: Arc<Mutex<ReadableChunks<Body>>>,
}

impl Deflate {
    fn new(stream: ReadableChunks<Body>) -> Self {
        let stream = Arc::new(Mutex::new(stream));
        let reader = stream.clone();
        Self {
            buf: BytesMut::with_capacity(INIT_BUFFER_SIZE),
            inner: Box::new(DeflateDecoder::new(Peeked::new(stream))),
            reader,
        }
    }
}

impl Stream for Deflate {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = self.buf.clone();
        if let Ok(mut reader) = self.reader.lock() {
            reader.waker = Some(cx.waker().clone());
        }
        poll_with_read(&mut self.inner, &mut buf)
    }
}

/// A `Read`able wrapper over a stream of chunks.
pub struct ReadableChunks<S> {
    state: ReadState,
    stream: S,
    waker: Option<Waker>,
}

enum ReadState {
    /// A chunk is ready to be read from.
    Ready(Bytes),
    /// The next chunk isn't ready yet.
    NotReady,
    /// The stream has finished.
    Eof,
    /// Stream is in err
    Error(hyper::Error),
}

#[derive(Debug)]
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
    inner: Arc<Mutex<R>>,
}

enum PeekedState {
    /// The internal buffer hasn't filled yet.
    NotReady,
    /// The internal buffer can be read.
    Ready(usize),
}

impl<R> Peeked<R> {
    #[inline]
    fn new(inner: Arc<Mutex<R>>) -> Self {
        Peeked {
            state: PeekedState::NotReady,
            peeked_buf: [0; 10],
            inner,
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
                    let buf = &mut self.peeked_buf[self.pos..];
                    let stream = self.inner.clone();
                    let mut reader = stream.lock().unwrap();
                    let read = reader.read(buf);

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
            stream,
            waker: None,
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
    S: Stream<Item = Result<Bytes, hyper::Error>> + std::marker::Unpin,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let waker = self.waker.as_ref().unwrap().clone();
        let mut cx = Context::from_waker(&waker);

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
                ReadState::NotReady => match self.poll_stream(&mut cx) {
                    Poll::Ready(StreamState::HasMore) => continue,
                    Poll::Ready(StreamState::Eof) => return Ok(0),
                    Poll::Pending => return Err(io::ErrorKind::WouldBlock.into()),
                },
                ReadState::Eof => return Ok(0),
                ReadState::Error(ref err) => {
                    return Err(io::Error::new(io::ErrorKind::Other, err.to_string()))
                },
            }
            self.state = ReadState::NotReady;
            return Ok(ret);
        }
    }
}

impl<S> ReadableChunks<S>
where
    S: Stream<Item = Result<Bytes, hyper::Error>> + std::marker::Unpin,
{
    /// Poll the readiness of the inner reader.
    ///
    /// This function will update the internal state and return a simplified
    /// version of the `ReadState`.
    fn poll_stream(&mut self, cx: &mut Context<'_>) -> Poll<StreamState> {
        match Pin::new(&mut self.stream).poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                self.state = ReadState::Ready(chunk);

                Poll::Ready(StreamState::HasMore)
            },
            Poll::Ready(Some(Err(err))) => {
                self.state = ReadState::Error(err);

                Poll::Ready(StreamState::Eof)
            },
            Poll::Ready(None) => {
                self.state = ReadState::Eof;
                Poll::Ready(StreamState::Eof)
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

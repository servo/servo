/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Adapted from an implementation in reqwest.

/*!
A non-blocking response decoder.

The decoder wraps a stream of bytes and produces a new stream of decompressed bytes.
The decompressed bytes aren't guaranteed to align to the compressed ones.

If the response is plaintext then no additional work is carried out.
Bytes are just passed along.

If the response is gzip, deflate or brotli then the bytes are decompressed.
*/

use std::error::Error;
use std::fmt;
use std::io::{self};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use async_compression::tokio::bufread::{BrotliDecoder, GzipDecoder, ZlibDecoder, ZstdDecoder};
use bytes::{Bytes, BytesMut};
use futures::stream::Peekable;
use futures::task::{Context, Poll};
use futures::{Future, Stream};
use futures_util::StreamExt;
use headers::{ContentLength, HeaderMapExt};
use http_body_util::BodyExt;
use hyper::Response;
use hyper::body::Body;
use hyper::header::{CONTENT_ENCODING, HeaderValue, TRANSFER_ENCODING};
use log::warn;
use net_traits::DecoderType;
use servo_config::pref;
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio_util::io::StreamReader;

use crate::connector::BoxedBody;

pub const DECODER_BUFFER_SIZE: usize = 8192;

/// Marker wrapper for errors that originate from the network body stream
#[derive(Debug)]
pub struct BodyStreamError(pub Box<dyn Error + Send + Sync>);

impl fmt::Display for BodyStreamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for BodyStreamError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.as_ref())
    }
}

/// Normalize errors produced by the decompressors to `ErrorKind::InvalidData`
/// so that `http_loader` reports them as `NetworkError::DecompressionError`.
pub fn map_decode_error(err: io::Error) -> io::Error {
    if err.kind() == io::ErrorKind::InvalidData {
        return err;
    }

    let mut source: Option<&(dyn Error + 'static)> = err.get_ref().map(|e| e as _);
    while let Some(e) = source {
        if e.is::<BodyStreamError>() {
            return err;
        }

        source = match e.downcast_ref::<io::Error>() {
            Some(io_error) => io_error.get_ref().map(|e| e as _),
            None => e.source(),
        };
    }
    io::Error::new(io::ErrorKind::InvalidData, err)
}

/// A response decompressor over a non-blocking stream of bytes.
///
/// The inner decoder may be constructed asynchronously.
pub struct Decoder {
    inner: Inner,
    encoded_data: Arc<Mutex<Option<(BytesMut, DecoderType)>>>,
}

enum Inner {
    /// A `PlainText` decoder just returns the response content as is.
    PlainText(BodyStream),
    /// A `Gzip` decoder will uncompress the gzipped response content before returning it.
    Gzip(FramedRead<GzipDecoder<StreamReader<Peekable<BodyStream>, Bytes>>, BytesCodec>),
    /// A `Delfate` decoder will uncompress the inflated response content before returning it.
    Deflate(FramedRead<ZlibDecoder<StreamReader<Peekable<BodyStream>, Bytes>>, BytesCodec>),
    /// A `Brotli` decoder will uncompress the brotli-encoded response content before returning it.
    Brotli(FramedRead<BrotliDecoder<StreamReader<Peekable<BodyStream>, Bytes>>, BytesCodec>),
    /// A `Zstd` decoder will uncompress the zstd-encoded response content before returning it.
    Zstd(FramedRead<ZstdDecoder<StreamReader<Peekable<BodyStream>, Bytes>>, BytesCodec>),
    /// A decoder that doesn't have a value yet.
    Pending(Pending),
}

/// A future attempt to poll the response body for EOF so we know whether to use gzip or not.
struct Pending {
    body: Peekable<BodyStream>,
    type_: DecoderType,
}

impl fmt::Debug for Decoder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Decoder").finish()
    }
}

impl Decoder {
    /// Returns the buffer the encoded bytes will be stored in.
    /// Originally this will be `None`. After the decoder is polled to completion (via the Stream impl), this can contain two values.
    /// Either `None`, meaning the decoded bytes and encoded bytes are equal. Or the encoded bytes.
    pub fn encoded_bytes(&self) -> Arc<Mutex<Option<(BytesMut, DecoderType)>>> {
        self.encoded_data.clone()
    }

    /// A plain text decoder.
    ///
    /// This decoder will emit the underlying bytes as-is.
    #[inline]
    pub fn plain_text(
        body: BoxedBody,
        is_secure_scheme: bool,
        content_length: Option<ContentLength>,
    ) -> Decoder {
        let body = BodyStream::new(body, is_secure_scheme, content_length);
        let encoded_data = body.data.clone();
        Decoder {
            inner: Inner::PlainText(body),
            encoded_data,
        }
    }

    /// A pending decoder.
    ///
    /// This decoder will buffer and decompress bytes that are encoded in the expected format.
    #[inline]
    pub fn pending(
        body: BoxedBody,
        type_: DecoderType,
        is_secure_scheme: bool,
        content_length: Option<ContentLength>,
    ) -> Decoder {
        let body = BodyStream::new(body, is_secure_scheme, content_length);
        let encoded_data = body.data.clone();
        Decoder {
            inner: Inner::Pending(Pending {
                body: body.peekable(),
                type_,
            }),
            encoded_data,
        }
    }

    /// Constructs a Decoder from a hyper response.
    ///
    /// A decoder is just a wrapper around the hyper response that knows
    /// how to decode the content body of the response.
    ///
    /// Uses the correct variant by inspecting the Content-Encoding header.
    pub fn detect(response: Response<BoxedBody>, is_secure_scheme: bool) -> Response<Decoder> {
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
                } else if enc == HeaderValue::from_static("zstd") {
                    Some(DecoderType::Zstd)
                } else {
                    None
                }
            })
        });
        let content_length = response.headers().typed_get::<ContentLength>();
        match decoder {
            Some(type_) => {
                response.map(|r| Decoder::pending(r, type_, is_secure_scheme, content_length))
            },
            None => response.map(|r| Decoder::plain_text(r, is_secure_scheme, content_length)),
        }
    }
}

impl Stream for Decoder {
    type Item = Result<Bytes, io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Do a read or poll for a pending decoder value.
        match self.inner {
            Inner::Pending(ref mut future) => match futures_core::ready!(Pin::new(future).poll(cx))
            {
                Ok(inner) => {
                    self.inner = inner;
                    self.poll_next(cx)
                },
                Err(e) => Poll::Ready(Some(Err(e))),
            },
            Inner::PlainText(ref mut body) => Pin::new(body).poll_next(cx),
            Inner::Gzip(ref mut decoder) => {
                match futures_core::ready!(Pin::new(decoder).poll_next(cx)) {
                    Some(Ok(bytes)) => Poll::Ready(Some(Ok(bytes.freeze()))),
                    Some(Err(err)) => Poll::Ready(Some(Err(map_decode_error(err)))),
                    None => Poll::Ready(None),
                }
            },
            Inner::Brotli(ref mut decoder) => {
                match futures_core::ready!(Pin::new(decoder).poll_next(cx)) {
                    Some(Ok(bytes)) => Poll::Ready(Some(Ok(bytes.freeze()))),
                    Some(Err(err)) => Poll::Ready(Some(Err(map_decode_error(err)))),
                    None => Poll::Ready(None),
                }
            },
            Inner::Deflate(ref mut decoder) => {
                match futures_core::ready!(Pin::new(decoder).poll_next(cx)) {
                    Some(Ok(bytes)) => Poll::Ready(Some(Ok(bytes.freeze()))),
                    Some(Err(err)) => Poll::Ready(Some(Err(map_decode_error(err)))),
                    None => Poll::Ready(None),
                }
            },
            Inner::Zstd(ref mut decoder) => {
                match futures_core::ready!(Pin::new(decoder).poll_next(cx)) {
                    Some(Ok(bytes)) => Poll::Ready(Some(Ok(bytes.freeze()))),
                    Some(Err(err)) => Poll::Ready(Some(Err(map_decode_error(err)))),
                    None => Poll::Ready(None),
                }
            },
        }
    }
}

impl Future for Pending {
    type Output = Result<Inner, io::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // In the event that we have a compressed stream, we want our first poll to be already written to the data. The first poll will be executed by poll_peek, so we have to do this first.
        // If we are in plaintext mode, we will disarm the body.
        // This method gets only called a couple of times.

        if !pref!(network_http_disk_cache).is_empty() || cfg!(feature = "test-util") {
            self.body.get_ref().arm_copy_mechanism(self.type_.clone());
        }

        match futures_core::ready!(Pin::new(&mut self.body).poll_peek(cx)) {
            Some(Ok(_)) => {
                // fallthrough
            },
            Some(Err(_e)) => {
                // error was just a ref, so we need to really poll to move it
                return Poll::Ready(Err(futures_core::ready!(
                    Pin::new(&mut self.body).poll_next(cx)
                )
                .expect("just peeked Some")
                .unwrap_err()));
            },
            None => {
                self.body.get_ref().disarm_copy_mechanism();
                return Poll::Ready(Ok(Inner::PlainText(BodyStream::empty(
                    self.body.get_ref().data.clone(),
                ))));
            },
        };

        let data = self.body.get_ref().data.clone();
        let body = std::mem::replace(&mut self.body, BodyStream::empty(data).peekable());

        let value = match self.type_ {
            DecoderType::Brotli => Inner::Brotli(FramedRead::with_capacity(
                BrotliDecoder::new(StreamReader::new(body)),
                BytesCodec::new(),
                DECODER_BUFFER_SIZE,
            )),
            DecoderType::Gzip => Inner::Gzip(FramedRead::with_capacity(
                GzipDecoder::new(StreamReader::new(body)),
                BytesCodec::new(),
                DECODER_BUFFER_SIZE,
            )),
            DecoderType::Deflate => Inner::Deflate(FramedRead::with_capacity(
                ZlibDecoder::new(StreamReader::new(body)),
                BytesCodec::new(),
                DECODER_BUFFER_SIZE,
            )),
            DecoderType::Zstd => Inner::Zstd(FramedRead::with_capacity(
                ZstdDecoder::new(StreamReader::new(body)),
                BytesCodec::new(),
                DECODER_BUFFER_SIZE,
            )),
        };

        Poll::Ready(Ok(value))
    }
}

/// This takes the BoxedBody and reads the bytes from it. We record the length because of rustls (see impl of Stream).
/// If the data field contains not None, we copy all (remaining) undecoded bytes into the data field.
struct BodyStream {
    body: BoxedBody,
    is_secure_scheme: bool,
    content_length: Option<ContentLength>,
    total_read: u64,
    data: Arc<Mutex<Option<(BytesMut, DecoderType)>>>,
}

impl BodyStream {
    fn empty(data: Arc<Mutex<Option<(BytesMut, DecoderType)>>>) -> Self {
        BodyStream {
            body: http_body_util::Empty::new()
                .map_err(|_| unreachable!())
                .boxed(),
            is_secure_scheme: false,
            content_length: None,
            total_read: 0,
            data,
        }
    }

    fn new(body: BoxedBody, is_secure_scheme: bool, content_length: Option<ContentLength>) -> Self {
        BodyStream {
            body,
            is_secure_scheme,
            content_length,
            total_read: 0,
            data: Arc::new(Mutex::new(None)),
        }
    }

    /// Arms the copy mechanism. All bytes that get received by the body stream will be also written to `self.data`
    fn arm_copy_mechanism(&self, decoder_type: DecoderType) {
        let mut data = self.data.lock().unwrap();
        if data.is_none() {
            *data = Some((BytesMut::new(), decoder_type));
        } else {
            warn!("You are arming the copy mechanism a second time")
        }
    }

    /// Disarms the copy mechanism and destroys all bytes that were already copied.
    fn disarm_copy_mechanism(&self) {
        *self.data.lock().unwrap() = None;
    }
}

impl Stream for BodyStream {
    type Item = Result<Bytes, io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match futures_core::ready!(Pin::new(&mut self.body).poll_frame(cx)) {
            Some(Ok(bytes)) => {
                let Ok(bytes) = bytes.into_data() else {
                    return Poll::Ready(None);
                };
                self.total_read += bytes.len() as u64;

                let mut data_lock = self.data.lock().unwrap();
                if let Some(data) = data_lock.as_mut() {
                    data.0.extend_from_slice(&bytes);
                }
                Poll::Ready(Some(Ok(bytes)))
            },
            Some(Err(err)) => {
                // To prevent truncation attacks rustls treats close connection without a close_notify as
                // an error of type std::io::Error with ErrorKind::UnexpectedEof.
                // https://docs.rs/rustls/latest/rustls/manual/_03_howto/index.html#unexpected-eof
                //
                // The error can be safely ignored if we known that all content was received or is explicitly
                // set in preferences.
                let all_content_read = self.content_length.is_some_and(|c| c.0 == self.total_read);
                if self.is_secure_scheme && all_content_read {
                    let source = err.source();
                    let is_unexpected_eof = source
                        .and_then(|e| e.downcast_ref::<io::Error>())
                        .is_some_and(|e| e.kind() == io::ErrorKind::UnexpectedEof);
                    if is_unexpected_eof {
                        return Poll::Ready(None);
                    }
                }
                Poll::Ready(Some(Err(io::Error::other(BodyStreamError(err.into())))))
            },
            None => Poll::Ready(None),
        }
    }
}

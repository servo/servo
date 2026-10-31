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
use std::sync::Arc;

use async_compression::tokio::bufread::{BrotliDecoder, GzipDecoder, ZlibDecoder, ZstdDecoder};
use bytes::Bytes;
use futures::stream::Peekable;
use futures::task::{Context, Poll};
use futures::{Future, Stream};
use futures_util::StreamExt;
use headers::{ContentLength, HeaderMapExt};
use http_body_util::BodyExt;
use hyper::Response;
use hyper::body::Body;
use hyper::header::{CONTENT_ENCODING, HeaderValue, TRANSFER_ENCODING};
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio_util::io::StreamReader;

use crate::connector::BoxedBody;
use crate::disk_cache::CacheStorage;
use crate::http_cache::{CacheKey, status_and_headers_is_cacheable};
use crate::http_loader::HttpState;

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
    inner: DecoderInner,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum DecoderType {
    Gzip,
    Brotli,
    Deflate,
    Zstd,
}

enum DecoderInner {
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
    Pending(PendingDecode),
}

/// A future attempt to poll the response body for EOF so we know whether to use gzip or not.
struct PendingDecode {
    body: Peekable<BodyStream>,
    type_: DecoderType,
}

impl fmt::Debug for Decoder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Decoder").finish()
    }
}

impl Decoder {
    /// A plain text decoder.
    ///
    /// This decoder will emit the underlying bytes as-is.
    #[inline]
    fn plain_text(
        body: BoxedBody,
        is_secure_scheme: bool,
        content_length: Option<ContentLength>,
        disk_cache: Option<CacheStorage>,
    ) -> Decoder {
        Decoder {
            inner: DecoderInner::PlainText(BodyStream::new(
                body,
                is_secure_scheme,
                content_length,
                disk_cache,
            )),
        }
    }

    /// A pending decoder.
    ///
    /// This decoder will buffer and decompress bytes that are encoded in the expected format.
    #[inline]
    fn pending(
        body: BoxedBody,
        type_: DecoderType,
        is_secure_scheme: bool,
        content_length: Option<ContentLength>,
        disk_cache: Option<CacheStorage>,
    ) -> Decoder {
        Decoder {
            inner: DecoderInner::Pending(PendingDecode {
                body: BodyStream::new(body, is_secure_scheme, content_length, disk_cache)
                    .peekable(),
                type_,
            }),
        }
    }

    /// Returns true if the content is encoded
    pub fn is_encoded(&self) -> bool {
        !matches!(self.inner, DecoderInner::PlainText(_))
    }

    /// Constructs a Decoder from a hyper response.
    ///
    /// A decoder is just a wrapper around the hyper response that knows
    /// how to decode the content body of the response.
    ///
    /// Uses the correct variant by inspecting the Content-Encoding header.
    pub fn detect(
        response: Response<BoxedBody>,
        is_secure_scheme: bool,
        should_store: bool,
        key: CacheKey,
        state: Arc<HttpState>,
    ) -> Response<Decoder> {
        let cacheable =
            status_and_headers_is_cacheable(&response.status(), response.headers()) && should_store;
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

        let disk_cache = if cacheable {
            Some(CacheStorage::new(key, state, decoder.into()))
        } else {
            None
        };
        match decoder {
            Some(type_) => response
                .map(|r| Decoder::pending(r, type_, is_secure_scheme, content_length, disk_cache)),
            None => response
                .map(|r| Decoder::plain_text(r, is_secure_scheme, content_length, disk_cache)),
        }
    }
}

impl Stream for Decoder {
    type Item = Result<Bytes, io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Do a read or poll for a pending decoder value.
        match self.inner {
            DecoderInner::Pending(ref mut future) => {
                match futures_core::ready!(Pin::new(future).poll(cx)) {
                    Ok(inner) => {
                        self.inner = inner;
                        self.poll_next(cx)
                    },
                    Err(e) => Poll::Ready(Some(Err(e))),
                }
            },
            DecoderInner::PlainText(ref mut body) => Pin::new(body).poll_next(cx),
            DecoderInner::Gzip(ref mut decoder) => {
                match futures_core::ready!(Pin::new(decoder).poll_next(cx)) {
                    Some(Ok(bytes)) => Poll::Ready(Some(Ok(bytes.freeze()))),
                    Some(Err(err)) => Poll::Ready(Some(Err(map_decode_error(err)))),
                    None => Poll::Ready(None),
                }
            },
            DecoderInner::Brotli(ref mut decoder) => {
                match futures_core::ready!(Pin::new(decoder).poll_next(cx)) {
                    Some(Ok(bytes)) => Poll::Ready(Some(Ok(bytes.freeze()))),
                    Some(Err(err)) => Poll::Ready(Some(Err(map_decode_error(err)))),
                    None => Poll::Ready(None),
                }
            },
            DecoderInner::Deflate(ref mut decoder) => {
                match futures_core::ready!(Pin::new(decoder).poll_next(cx)) {
                    Some(Ok(bytes)) => Poll::Ready(Some(Ok(bytes.freeze()))),
                    Some(Err(err)) => Poll::Ready(Some(Err(map_decode_error(err)))),
                    None => Poll::Ready(None),
                }
            },
            DecoderInner::Zstd(ref mut decoder) => {
                match futures_core::ready!(Pin::new(decoder).poll_next(cx)) {
                    Some(Ok(bytes)) => Poll::Ready(Some(Ok(bytes.freeze()))),
                    Some(Err(err)) => Poll::Ready(Some(Err(map_decode_error(err)))),
                    None => Poll::Ready(None),
                }
            },
        }
    }
}

/// Construct the inner decoder type
// These decoder all need to poll the body till it returns Ready(None), for the cache to call its finishing routine.
// That is the reason we "expect" multiple_members.
fn construct_inner(type_: DecoderType, body: Peekable<BodyStream>) -> DecoderInner {
    let stream_reader = StreamReader::new(body);

    match type_ {
        DecoderType::Brotli => {
            let mut brotli_decoder = BrotliDecoder::new(stream_reader);
            brotli_decoder.multiple_members(true);
            DecoderInner::Brotli(FramedRead::with_capacity(
                brotli_decoder,
                BytesCodec::new(),
                DECODER_BUFFER_SIZE,
            ))
        },

        DecoderType::Gzip => {
            let mut gzip_decoder = GzipDecoder::new(stream_reader);
            gzip_decoder.multiple_members(true);
            DecoderInner::Gzip(FramedRead::with_capacity(
                gzip_decoder,
                BytesCodec::new(),
                DECODER_BUFFER_SIZE,
            ))
        },
        DecoderType::Deflate => {
            let mut deflate_decoder = ZlibDecoder::new(stream_reader);
            deflate_decoder.multiple_members(true);
            DecoderInner::Deflate(FramedRead::with_capacity(
                deflate_decoder,
                BytesCodec::new(),
                DECODER_BUFFER_SIZE,
            ))
        },
        DecoderType::Zstd => {
            let mut zstd_decoder = ZstdDecoder::new(stream_reader);
            zstd_decoder.multiple_members(true);
            DecoderInner::Zstd(FramedRead::with_capacity(
                zstd_decoder,
                BytesCodec::new(),
                DECODER_BUFFER_SIZE,
            ))
        },
    }
}

impl Future for PendingDecode {
    type Output = Result<DecoderInner, io::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
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
            None => return Poll::Ready(Ok(DecoderInner::PlainText(BodyStream::empty()))),
        };

        let body = std::mem::replace(&mut self.body, BodyStream::empty().peekable());
        Poll::Ready(Ok(construct_inner(self.type_, body)))
    }
}

struct BodyStream {
    body: BoxedBody,
    is_secure_scheme: bool,
    content_length: Option<ContentLength>,
    total_read: u64,
    disk_cache: Option<CacheStorage>,
}

impl BodyStream {
    fn empty() -> Self {
        BodyStream {
            body: http_body_util::Empty::new()
                .map_err(|_| unreachable!())
                .boxed(),
            is_secure_scheme: false,
            content_length: None,
            total_read: 0,
            disk_cache: None,
        }
    }

    fn new(
        body: BoxedBody,
        is_secure_scheme: bool,
        content_length: Option<ContentLength>,
        disk_cache: Option<CacheStorage>,
    ) -> Self {
        BodyStream {
            body,
            is_secure_scheme,
            content_length,
            total_read: 0,
            disk_cache,
        }
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

                if let Some(disk_cache) = self.disk_cache.as_mut() {
                    disk_cache.send(&bytes);
                }
                self.total_read += bytes.len() as u64;
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
            None => {
                if let Some(disk_cache) = self.disk_cache.as_mut() {
                    disk_cache.finish();
                }
                Poll::Ready(None)
            },
        }
    }
}

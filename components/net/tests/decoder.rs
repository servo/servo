/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io::{self, Read};
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Bytes, BytesMut};
use flate2::Compression;
use flate2::read::GzEncoder;
use http_body_util::BodyExt;
use hyper::body::{Body, Frame};
use net::decoder::Decoder;
use net::test::{BodyStreamError, map_decode_error};
use net_traits::DecoderType;
use tokio_stream::StreamExt;

#[test]
fn test_map_decode_error_wraps_decoder_errors_as_invalid_data() {
    for err in [
        io::Error::other("Unknown frame"),
        io::Error::new(io::ErrorKind::UnexpectedEof, "zstd stream did not finish"),
    ] {
        assert_eq!(
            map_decode_error(err).kind(),
            io::ErrorKind::InvalidData,
            "decoder errors should be normalized to InvalidData"
        );
    }
}

#[test]
fn test_map_decode_error_passes_network_errors_through() {
    let network_error = io::Error::other(BodyStreamError("connection reset".into()));
    let mapped = map_decode_error(network_error);
    assert_eq!(mapped.kind(), io::ErrorKind::Other);
    assert!(
        mapped
            .get_ref()
            .is_some_and(|inner| inner.is::<BodyStreamError>())
    );
}

#[test]
fn test_map_decode_error_passes_nested_network_errors_through() {
    let network_error = io::Error::other(BodyStreamError("connection reset".into()));
    let wrapped = io::Error::new(io::ErrorKind::BrokenPipe, network_error);
    let mapped = map_decode_error(wrapped);
    assert_eq!(mapped.kind(), io::ErrorKind::BrokenPipe);
}

#[tokio::test]
/// Test if the bodystream gzip decoding preserves the encoded bytes if setup.
async fn test_bodystream_gzip_decoding() {
    struct MyBody {
        bytes: Bytes,
        done: bool,
    }

    impl Body for MyBody {
        type Data = Bytes;
        type Error = hyper::Error;

        // Required method
        fn poll_frame(
            mut self: Pin<&mut Self>,
            _: &mut Context<'_>,
        ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
            if self.done {
                Poll::Ready(None)
            } else if self.bytes.len() < 4 {
                self.done = true;
                Poll::Ready(Some(Ok(Frame::data(self.bytes.clone()))))
            } else {
                let old_data = self.bytes.split_off(4);
                let new_data = self.bytes.clone();
                self.bytes = old_data;

                Poll::Ready(Some(Ok(Frame::data(new_data))))
            }
        }
    }

    let teststring = "This is a big test string that we are going to compress. \nI am making this a bit longer than normal to test hopefully not just the first frame";

    let mut compressed_bytes = Vec::new();
    let mut gz = GzEncoder::new(teststring.as_bytes(), Compression::fast());
    gz.read_to_end(&mut compressed_bytes).unwrap();

    let bytes = Bytes::copy_from_slice(&compressed_bytes);
    let body = MyBody { bytes, done: false };

    let decoder = Decoder::pending(body.boxed(), DecoderType::Gzip, false, None);
    let encoded_bytes = decoder.encoded_bytes();
    let output = decoder
        .fold(BytesMut::new(), |mut acc, data| {
            if let Ok(data) = data {
                acc.extend_from_slice(&data);
            }
            acc
        })
        .await;
    assert_eq!(Bytes::copy_from_slice(teststring.as_bytes()), output);

    let encoded_bytes = encoded_bytes.lock().unwrap();
    assert!(encoded_bytes.is_some());
    assert_eq!(compressed_bytes, encoded_bytes.as_ref().unwrap().0);
}

#[tokio::test]
/// Test if the bodystream plain decoding does not preserve the encoded bytes
async fn test_bodystream_plain_decoding() {
    struct MyBody {
        bytes: Bytes,
        done: bool,
    }

    impl Body for MyBody {
        type Data = Bytes;
        type Error = hyper::Error;

        // Required method
        fn poll_frame(
            mut self: Pin<&mut Self>,
            _: &mut Context<'_>,
        ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
            if self.done {
                Poll::Ready(None)
            } else if self.bytes.len() < 4 {
                self.done = true;
                Poll::Ready(Some(Ok(Frame::data(self.bytes.clone()))))
            } else {
                let old_data = self.bytes.split_off(4);
                let new_data = self.bytes.clone();
                self.bytes = old_data;

                Poll::Ready(Some(Ok(Frame::data(new_data))))
            }
        }
    }

    let teststring = "This is a big test string that we are going to compress. \nI am making this a bit longer than normal to test hopefully not just the first frame";

    let bytes = Bytes::copy_from_slice(&teststring.as_bytes());
    let body = MyBody { bytes, done: false };

    let decoder = Decoder::plain_text(body.boxed(), false, None);
    let encoded_bytes = decoder.encoded_bytes();
    let output = decoder
        .fold(BytesMut::new(), |mut acc, data| {
            if let Ok(data) = data {
                acc.extend_from_slice(&data);
            }
            acc
        })
        .await;
    assert_eq!(Bytes::copy_from_slice(teststring.as_bytes()), output);

    let encoded_bytes = encoded_bytes.lock().unwrap();
    assert!(encoded_bytes.is_none());
}

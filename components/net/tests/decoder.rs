/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io;

use net::test::{BodyStreamError, map_decode_error};

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

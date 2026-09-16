/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Helper types for rustls::ProtocolVersion and rustls::NamedGroup.

use malloc_size_of_derive::MallocSizeOf;
use rustls::{NamedGroup, ProtocolVersion};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, MallocSizeOf)]
#[non_exhaustive]
/// This is a clone of rustls::ProtocolVersion because it currently does not support serde plus a NotYetImplemented flag.
pub enum ServoProtocolVersion {
    SSLv2,
    SSLv3,
    TLSv1_0,
    TLSv1_1,
    TLSv1_2,
    TLSv1_3,
    DTLSv1_0,
    DTLSv1_2,
    DTLSv1_3,
    Unknown(u16),
    NotYetImplemented,
}

impl From<ProtocolVersion> for ServoProtocolVersion {
    fn from(value: ProtocolVersion) -> Self {
        match value {
            ProtocolVersion::SSLv2 => Self::SSLv2,
            ProtocolVersion::SSLv3 => Self::SSLv3,
            ProtocolVersion::TLSv1_0 => Self::TLSv1_0,
            ProtocolVersion::TLSv1_1 => Self::TLSv1_1,
            ProtocolVersion::TLSv1_2 => Self::TLSv1_2,
            ProtocolVersion::TLSv1_3 => Self::TLSv1_3,
            ProtocolVersion::DTLSv1_0 => Self::DTLSv1_0,
            ProtocolVersion::DTLSv1_2 => Self::DTLSv1_2,
            ProtocolVersion::DTLSv1_3 => Self::DTLSv1_3,
            ProtocolVersion::Unknown(value) => Self::Unknown(value),
            _ => Self::NotYetImplemented,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, MallocSizeOf)]
#[non_exhaustive]
/// This is essentially a clone of rustls::ServoNamedGroup because it currently does not support serde plus a NotYetImplemented flag.
pub enum ServoNamedGroup {
    Secp256r1,
    Secp384r1,
    Secp521r1,
    X25519,
    X448,
    FFDHE2048,
    FFDHE3072,
    FFDHE4096,
    FFDHE6144,
    FFDHE8192,
    MLKEM512,
    MLKEM768,
    MLKEM1024,
    Secp256r1MLKEM768,
    X25519MLKEM768,
    Unknown(u16),
    NotYetImplemented,
}

impl From<NamedGroup> for ServoNamedGroup {
    fn from(value: NamedGroup) -> Self {
        match value {
            NamedGroup::secp256r1 => Self::Secp256r1,
            NamedGroup::secp384r1 => Self::Secp384r1,
            NamedGroup::secp521r1 => Self::Secp521r1,
            NamedGroup::X25519 => Self::X25519,
            NamedGroup::X448 => Self::X448,
            NamedGroup::FFDHE2048 => Self::FFDHE2048,
            NamedGroup::FFDHE3072 => Self::FFDHE3072,
            NamedGroup::FFDHE4096 => Self::FFDHE4096,
            NamedGroup::FFDHE6144 => Self::FFDHE6144,
            NamedGroup::FFDHE8192 => Self::FFDHE8192,
            NamedGroup::MLKEM512 => Self::MLKEM512,
            NamedGroup::MLKEM768 => Self::MLKEM768,
            NamedGroup::MLKEM1024 => Self::MLKEM1024,
            NamedGroup::secp256r1MLKEM768 => Self::Secp256r1MLKEM768,
            NamedGroup::X25519MLKEM768 => Self::X25519MLKEM768,
            NamedGroup::Unknown(value) => Self::Unknown(value),
            _ => Self::NotYetImplemented,
        }
    }
}

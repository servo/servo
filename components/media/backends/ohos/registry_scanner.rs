/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use once_cell::sync::Lazy;

pub static OHOS_REGISTRY_SCANNER: Lazy<OhosRegistryScanner> =
    Lazy::new(|| OhosRegistryScanner::new());

// Should be a combination of mime/codecs
// If the type we are matching only contain mime, then we only match the container.
//
pub struct OhosRegistryScanner {
    av_player_supported_mime_codecs_type: HashMap<&'static str, &'static [&'static str]>,
}

impl OhosRegistryScanner {
    fn new() -> OhosRegistryScanner {
        let mut registry_scanner = OhosRegistryScanner {
            av_player_supported_mime_codecs_type: HashMap::new(),
        };
        registry_scanner.initialize_av_player_container_and_codecs();
        registry_scanner
    }

    pub fn are_mime_and_codecs_supported(&self, container_type: &str, codecs: &Vec<&str>) -> bool {
        let Some(supported_codecs) = self
            .av_player_supported_mime_codecs_type
            .get(container_type)
        else {
            return false;
        };
        codecs.iter().all(|codec| {
            supported_codecs.contains(codec) || {
                supported_codecs.iter().any(|supported_codec| {
                    if let Some(stripped) = supported_codec.strip_suffix('*') {
                        if codec.starts_with(stripped) {
                            return true;
                        }
                    }
                    false
                })
            }
        })
    }

    fn initialize_av_player_container_and_codecs(&mut self) {
        // Video Container
        self.av_player_supported_mime_codecs_type
            .insert("video/mp4", &["hev1*", "hvc1*", "aac", "mp3", "avc*"]);
        self.av_player_supported_mime_codecs_type
            .insert("video/mkv", &["hev1*", "hvc1*", "aac", "mp3", "avc*"]);
        self.av_player_supported_mime_codecs_type
            .insert("video/ts", &["hev1*", "hvc1*", "aac", "mp3", "avc*"]);
        // Audio Container
        self.av_player_supported_mime_codecs_type
            .insert("audio/m4a", &["aac"]);
        self.av_player_supported_mime_codecs_type
            .insert("audio/aac", &["aac"]);
        self.av_player_supported_mime_codecs_type
            .insert("audio/mp3", &["mp3"]);
        self.av_player_supported_mime_codecs_type
            .insert("audio/ogg", &["vorbis"]);
        self.av_player_supported_mime_codecs_type
            .insert("audio/wav", &["1", "audio/pcm"]);
        self.av_player_supported_mime_codecs_type
            .insert("audio/flac", &["flac"]);
        self.av_player_supported_mime_codecs_type
            .insert("audio/amr", &["amr"]);
        self.av_player_supported_mime_codecs_type
            .insert("audio/ape", &["ape"]);
    }
}

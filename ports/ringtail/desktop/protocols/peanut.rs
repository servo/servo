/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::future::Future;
use std::io::Read;
use std::pin::Pin;
use std::time::Duration;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use flate2::read::GzDecoder;
use headers::{ContentType, HeaderMapExt};
use serde::{Deserialize, Serialize};
use servo::protocol_handler::{
    DoneChannel, FetchContext, NetworkError, ProtocolHandler, Request, ResourceFetchTiming,
    Response, ResponseBody,
};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::time::timeout;

const STATIC_DNS_URL: &str = "https://voxelite.neocities.org/ringtail/dns/dns.json";

// Fixed pre-shared key for encryption (32 bytes for AES-256)
const PRE_SHARED_KEY: &[u8; 32] = b"PeanutProtocolFixedKey1234567890";

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PeanutDnsRecord {
    pub server_address: String, 
}

#[derive(Default)]
pub struct PeanutProtocolHandler {}

impl ProtocolHandler for PeanutProtocolHandler {
    fn privileged_paths(&self) -> &'static [&'static str] {
        &[]
    }

    fn is_fetchable(&self) -> bool {
        true
    }

    fn load(
        &self,
        request: &mut Request,
        _done_chan: &mut DoneChannel,
        _context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let url = request.current_url().clone();
        let timing_type = request.timing_type();

        Box::pin(async move {
            let domain = url.host_str().unwrap_or("");
            let requested_file = url.path().trim_start_matches('/').to_string();

            if domain.is_empty() || requested_file.is_empty() {
                return Response::network_error(NetworkError::ResourceLoadError(
                    "Invalid URL format. Expected peanut://domain/file".to_string()
                ));
            }

            // 1. Fetch the static JSON list map from the HTTP server
            let dns_map: HashMap<String, PeanutDnsRecord> = match reqwest::get(STATIC_DNS_URL).await {
                Ok(res) => {
                    if res.status().is_success() {
                        match res.json::<HashMap<String, PeanutDnsRecord>>().await {
                            Ok(map) => map,
                            Err(_) => return Response::network_error(NetworkError::ResourceLoadError("Malformed Static DNS JSON payload".to_string())),
                        }
                    } else {
                        return Response::network_error(NetworkError::ResourceLoadError("Static DNS HTTP registry unavailable".to_string()));
                    }
                }
                Err(e) => return Response::network_error(NetworkError::ResourceLoadError(format!("Could not fetch DNS list: {}", e))),
            };

            // 2. Perform registry lookup for domain name mapping
            let dns_record = match dns_map.get(domain) {
                Some(record) => record,
                None => return Response::network_error(NetworkError::ResourceLoadError(format!("Domain '{}' not found in static DNS list", domain))),
            };

            // 3. Connect via raw TCP stream and handle custom E2EE payload decryption
            let archive_bytes = match timeout(Duration::from_secs(20), async {
                // Open raw TCP connection
                let mut stream = TcpStream::connect(&dns_record.server_address).await
                    .map_err(|e| format!("TCP connection failed: {}", e))?;

                // Read the whole encrypted response bundle
                let mut encrypted_buffer = Vec::new();
                stream.read_to_end(&mut encrypted_buffer).await
                    .map_err(|e| format!("Failed reading stream payload: {}", e))?;
                
                if encrypted_buffer.len() < 12 {
                    return Err("Received payload is too short to contain a valid nonce structure".to_string());
                }

                // Split out the structural protocol components: [12 Byte Nonce] [Ciphertext + 16 Byte Tag]
                let (nonce_bytes, ciphertext) = encrypted_buffer.split_at(12);

                // Instantiate AES-GCM cipher block using the pre-shared key
                let cipher = Aes256Gcm::new_from_slice(PRE_SHARED_KEY)
                    .map_err(|e| format!("Cipher initialization failure: {}", e))?;
                let nonce = Nonce::from_slice(nonce_bytes);

                // Decrypt into the target uncompressed tarball binary content
                let decrypted_bytes = cipher.decrypt(nonce, ciphertext)
                    .map_err(|e| format!("E2EE Authenticated Decryption Tag mismatch / check failure: {}", e))?;

                Ok::<Vec<u8>, String>(decrypted_bytes)
            })
            .await
            {
                Ok(Ok(data)) => data,
                Ok(Err(e)) => return Response::network_error(NetworkError::ResourceLoadError(e)),
                Err(_) => return Response::network_error(NetworkError::ResourceLoadError("Stream data retrieval timeout reached".to_string())),
            };

            // 4. Extract target index data from uncompressed decrypted tarball
            let mut file_bytes = Vec::new();
            let decoder = GzDecoder::new(&archive_bytes[..]);
            let mut archive = tar::Archive::new(decoder);
            let entries = match archive.entries() {
                Ok(e) => e,
                Err(_) => return Response::network_error(NetworkError::ResourceLoadError("Invalid payload data envelope structure".to_string())),
            };

            let mut found = false;
            for entry_result in entries {
                if let Ok(mut entry) = entry_result {
                    if let Ok(path) = entry.path() {
                        let path_str = path.to_string_lossy();
                        if path_str.trim_start_matches('/') == requested_file {
                            if entry.read_to_end(&mut file_bytes).is_err() {
                                return Response::network_error(NetworkError::ResourceLoadError("Extraction read processing crash".to_string()));
                            }
                            found = true;
                            break;
                        }
                    }
                }
            }

            if !found {
                return Response::network_error(NetworkError::ResourceLoadError("File component missing inside directory container".to_string()));
            }

            let mut response = Response::new(url, ResourceFetchTiming::new(timing_type));
            response.headers.typed_insert(guess_content_type(&requested_file));
            *response.body.lock() = ResponseBody::Done(file_bytes);
            response
        })
    }
}

fn guess_content_type(file_path: &str) -> ContentType {
    match file_path.rsplit('.').next() {
        Some("html") | Some("htm") => ContentType::html(),
        Some("css") => ContentType::from(mime_guess::mime::TEXT_CSS),
        Some("js") => ContentType::from(mime_guess::mime::APPLICATION_JAVASCRIPT),
        Some("json") => ContentType::json(),
        Some("png") => ContentType::from(mime_guess::mime::IMAGE_PNG),
        Some("jpg") | Some("jpeg") => ContentType::from(mime_guess::mime::IMAGE_JPEG),
        Some("svg") => ContentType::from(mime_guess::mime::IMAGE_SVG),
        _ => ContentType::octet_stream(),
    }
}
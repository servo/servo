/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

pub mod async_runtime;
pub mod connector;
pub mod cookie;
pub mod cookie_storage;
mod decoder;
mod devtools;
mod disk_cache;
pub mod embedder;
pub mod filemanager_thread;
mod hosts;
pub mod hsts;
pub mod http_cache;
pub mod http_loader;
pub mod image_cache;
pub mod download_hook {
    //! 观测到附件响应时通知嵌入方。
    //!
    //! Servo 自身没有下载管理器，嵌在浏览器里时由嵌入方负责真正落盘。
    use std::sync::RwLock;

    type Sink = Box<dyn Fn(String) + Send + Sync>;

    static SINK: RwLock<Option<Sink>> = RwLock::new(None);

    /// 注册接收函数（只会被 servoshell 调用一次）。
    pub fn set_sink(sink: Sink) {
        *SINK.write().unwrap() = Some(sink);
    }

    /// 报告一个附件下载地址，附带服务端建议的文件名（来自 `Content-Disposition`）。
    ///
    /// 载荷格式为 `<文件名>\u{1f}<url>`；没有文件名时只发 url。
    pub fn notify(url: &str, filename: Option<&str>, is_navigation: bool) {
        // 载荷：<导航标记>\u{1f}<文件名>\u{1f}<url>
        // 子资源也可能带 Content-Disposition，那种情况绝不能把页面退回上一页。
        let payload = format!(
            "{}\u{1f}{}\u{1f}{url}",
            if is_navigation { "1" } else { "0" },
            filename.unwrap_or("")
        );
        if let Some(sink) = SINK.read().unwrap().as_ref() {
            sink(payload);
        }
    }

    /// 顶层导航遇到这些类型时浏览器不会渲染，直接按下载处理。
    /// 与 Chromium 的默认下载类型表保持一致（只做前缀匹配）。
    pub fn is_download_content_type(content_type: &str) -> bool {
        const TYPES: &[&str] = &[
            "application/octet-stream",
            "application/force-download",
            "application/zip",
            "application/x-zip-compressed",
            "application/x-7z-compressed",
            "application/x-rar-compressed",
            "application/gzip",
            "application/x-tar",
            "application/x-bzip2",
            "application/x-xz",
            "application/vnd.android.package-archive",
            "application/epub+zip",
            "application/pdf",
            "application/msword",
            "application/vnd.ms-excel",
            "application/vnd.ms-powerpoint",
            "application/vnd.openxmlformats-officedocument",
            "application/x-iso9660-image",
            "application/x-apple-diskimage",
            "application/vnd.debian.binary-package",
            "application/x-rpm",
        ];
        TYPES.iter().any(|prefix| content_type.starts_with(prefix))
    }

    /// 从 `Content-Disposition` 里取出文件名，优先 RFC 5987 的 `filename*`。
    pub fn parse_filename(value: &str) -> Option<String> {
        let lower = value.to_ascii_lowercase();
        if let Some(pos) = lower.find("filename*=") {
            let rest = &value[pos + "filename*=".len()..];
            let encoded = match rest.find("''") {
                Some(index) => &rest[index + 2..],
                None => rest,
            };
            let end = encoded.find(';').unwrap_or(encoded.len());
            let decoded = percent_decode(encoded[..end].trim().trim_matches('"'));
            if !decoded.is_empty() {
                return Some(decoded);
            }
        }
        if let Some(pos) = lower.find("filename=") {
            let rest = &value[pos + "filename=".len()..];
            let end = rest.find(';').unwrap_or(rest.len());
            let cleaned = rest[..end].trim().trim_matches('"');
            if !cleaned.is_empty() {
                return Some(cleaned.to_string());
            }
        }
        None
    }

    /// 极简的百分号解码（只处理 UTF-8 百分号编码）。
    fn percent_decode(input: &str) -> String {
        let bytes = input.as_bytes();
        let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'%' && index + 2 < bytes.len() {
                let hex = &input[index + 1..index + 3];
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte);
                    index += 3;
                    continue;
                }
            }
            out.push(bytes[index]);
            index += 1;
        }
        String::from_utf8_lossy(&out).into_owned()
    }
}

pub mod local_directory_listing;
pub mod protocols;
pub mod request_interceptor;
pub mod resource_thread;
pub mod subresource_integrity;
#[cfg(feature = "test-util")]
pub mod test_util;
mod websocket_loader;

/// An implementation of the [Fetch specification](https://fetch.spec.whatwg.org/)
pub mod fetch {
    pub mod cors_cache;
    pub mod fetch_params;
    pub mod headers;
    pub mod methods;
}

/// A module for re-exports of items used in unit tests.
pub mod test {
    pub use crate::decoder::{BodyStreamError, DECODER_BUFFER_SIZE, map_decode_error};
    pub use crate::hosts::parse_hostsfile;
    pub use crate::http_loader::HttpState;
}

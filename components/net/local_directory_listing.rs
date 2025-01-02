/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fs::{DirEntry, Metadata, ReadDir};
use std::path::PathBuf;

use chrono::{DateTime, Local};
use embedder_traits::resources::{read_string, Resource};
use headers::{ContentType, HeaderMapExt};
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};
use net_traits::{NetworkError, ResourceFetchTiming};
use servo_config::pref;
use servo_url::ServoUrl;
use url::Url;

pub fn fetch(request: &mut Request, url: ServoUrl, path_buf: PathBuf) -> Response {
    if !pref!(network.local_directory_listing.enabled) {
        // If you want to be able to browse local directories, configure Servo prefs so that
        // "network.local_directory_listing.enabled" is set to true.
        return Response::network_error(NetworkError::Internal(
            "Local directory listing feature has not been enabled in preferences".into(),
        ));
    }

    if !request.origin.is_opaque() {
        // Checking for an opaque origin as a shorthand for user activation
        // as opposed to a request originating from a script.
        // TODO(32534): carefully consider security of this approach.
        return Response::network_error(NetworkError::Internal(
            "Cannot request local directory listing from non-local origin.".into(),
        ));
    }

    let directory_contents = match std::fs::read_dir(path_buf.clone()) {
        Ok(directory_contents) => directory_contents,
        Err(error) => {
            return Response::network_error(NetworkError::Internal(format!(
                "Unable to access directory: {error}"
            )));
        },
    };

    let output = build_html_directory_listing(url.as_url(), path_buf, directory_contents);

    let mut response = Response::new(url, ResourceFetchTiming::new(request.timing_type()));
    response.headers.typed_insert(ContentType::html());
    *response.body.lock().unwrap() = ResponseBody::Done(output.into_bytes());

    response
}

/// Returns an the string of an JavaScript `<script>` tag calling the `setData` function with the
/// contents of the given [`ReadDir`] directory listing.
///
/// # Arguments
///
/// * `url` - the original URL of the request that triggered this directory listing.
/// * `path` - the full path to the local directory.
/// * `directory_contents` - a [`ReadDir`] with the contents of the directory.
pub fn build_html_directory_listing(
    url: &Url,
    path: PathBuf,
    directory_contents: ReadDir,
) -> String {
    let mut page_html = String::with_capacity(1024);
    page_html.push_str("<!DOCTYPE html>");

    let mut parent_url_string = String::new();
    if path.parent().is_some() {
        let mut parent_url = url.clone();
        if let Ok(mut path_segments) = parent_url.path_segments_mut() {
            path_segments.pop();
        }
        parent_url.as_str().clone_into(&mut parent_url_string);
    }

    page_html.push_str(&read_string(Resource::DirectoryListingHTML));

    page_html.push_str("<script>\n");
    page_html.push_str(&format!(
        "setData({:?}, {:?}, [",
        url.as_str(),
        parent_url_string
    ));

    for directory_entry in directory_contents {
        let Ok(directory_entry) = directory_entry else {
            continue;
        };
        let Ok(metadata) = directory_entry.metadata() else {
            continue;
        };
        write_directory_entry(directory_entry, metadata, url, &mut page_html);
    }

    page_html.push_str("]);");
    page_html.push_str("</script>\n");

    page_html
}

fn write_directory_entry(entry: DirEntry, metadata: Metadata, url: &Url, output: &mut String) {
    let Ok(name) = entry.file_name().into_string() else {
        return;
    };

    let mut file_url = url.clone();
    {
        let Ok(mut path_segments) = file_url.path_segments_mut() else {
            return;
        };
        path_segments.push(&name);
    }

    let class = if metadata.is_dir() {
        "directory"
    } else if metadata.is_symlink() {
        "symlink"
    } else {
        "file"
    };

    let file_url_string = &file_url.to_string();
    let file_size = metadata_to_file_size_string(&metadata);
    let last_modified = metadata
        .modified()
        .map(DateTime::<Local>::from)
        .map(|time| time.format("%F %r").to_string())
        .unwrap_or_default();

    output.push_str(&format!(
        "[{class:?}, {name:?}, {file_url_string:?}, {file_size:?}, {last_modified:?}],"
    ));
}

pub fn metadata_to_file_size_string(metadata: &Metadata) -> String {
    if !metadata.is_file() {
        return String::new();
    }

    let mut float_size = metadata.len() as f64;
    let mut prefix_power = 0;
    while float_size > 1000.0 && prefix_power < 3 {
        float_size /= 1000.0;
        prefix_power += 1;
    }

    let prefix = match prefix_power {
        0 => "B",
        1 => "KB",
        2 => "MB",
        _ => "GB",
    };

    format!("{:.2} {prefix}", float_size)
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

use embedder_traits::resources::{read_string, Resource};
use headers::{ContentType, HeaderMapExt};
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};
use net_traits::{NetworkError, ResourceFetchTiming};
use servo_config::pref;
use servo_url::ServoUrl;
use time_03::OffsetDateTime;
use url::Url;

pub fn is_request_allowed(request: &Request) -> Result<(), Response> {
    if !pref!(network.local_directory_listing.enabled) {
        // If you want to be able to browse local directories, configure Servo prefs so that
        // "network.local_directory_listing.enabled" is set to true.
        Err(Response::network_error(NetworkError::Internal(
            "Local directory listing feature has not been enabled in preferences".into(),
        )))
    } else if !request.origin.is_opaque() {
        // Checking for an opaque origin as a shorthand for user activation
        // as opposed to a request originating from a script.
        // TODO: Raise and link to GitHub issue to request consideration of this concern.
        Err(Response::network_error(NetworkError::Internal(
            "Cannot request local directory listing from non-local origin.".to_string(),
        )))
    } else {
        Ok(())
    }
}

pub fn fetch(
    request: &mut Request,
    url: ServoUrl,
    directory: impl Into<DirectorySummary>,
) -> Response {
    let url = if !url.path().ends_with('/') {
        // Re-read the path as a directory, so that Servo adds the
        // forward-slash to the end of the URL (at least internally,
        // though the URL bar does not reflect this) and loads the
        // linked directories and files correctly when clicked.
        if let Ok(dir_url) = Url::from_directory_path(url.path()) {
            ServoUrl::from_url(dir_url)
        } else {
            return Response::network_error(NetworkError::Internal(format!(
                "Unable to parse local directory path {}",
                url
            )));
        }
    } else {
        url
    };
    let mut response = Response::new(url, ResourceFetchTiming::new(request.timing_type()));
    response.headers.typed_insert(ContentType::html());
    let directory_summary = directory.into();
    let page_text = build_html_directory_listing(directory_summary);
    let bytes: Vec<u8> = page_text.into_bytes();
    *response.body.lock().unwrap() = ResponseBody::Done(bytes);
    response
}

impl Into<DirectorySummary> for PathBuf {
    fn into(self) -> DirectorySummary {
        let path = self
            .to_str()
            .map(str::to_string)
            .ok_or("Invalid directory path.");
        let has_parent = self.parent().is_some();
        let items = if let Ok(entries) = std::fs::read_dir(self) {
            Ok(gather_directory_items(entries))
        } else {
            Err("Unable to iterate directory contents.")
        };
        DirectorySummary {
            path,
            has_parent,
            items,
        }
    }
}

fn gather_directory_items(
    entries: std::fs::ReadDir,
) -> BTreeMap<OsString, DirectoryItemDescriptor> {
    let mut items = BTreeMap::new();
    for entry in entries {
        if let Ok(entry) = entry {
            if let Ok(meta) = entry.metadata() {
                let os_name = entry.file_name();
                let entry_name = os_name.to_str().map(str::to_string);
                if meta.is_dir() {
                    items.insert(
                        os_name,
                        DirectoryItemDescriptor {
                            item_type: DirectoryItemType::SubDirectory,
                            name: entry_name,
                            size: None,
                            last_modified: Some(meta.modified().map(|m| m.into())),
                        },
                    );
                } else if meta.is_file() || meta.is_symlink() {
                    items.insert(
                        os_name,
                        DirectoryItemDescriptor {
                            item_type: match meta.is_symlink() {
                                true => DirectoryItemType::Symlink,
                                false => DirectoryItemType::File,
                            },
                            name: entry_name,
                            size: Some(meta.len()),
                            last_modified: Some(meta.modified().map(|m| m.into())),
                        },
                    );
                }
            }
        }
    }
    items
}

pub struct DirectorySummary {
    pub path: Result<String, &'static str>,
    pub has_parent: bool,
    pub items: Result<BTreeMap<OsString, DirectoryItemDescriptor>, &'static str>,
}

pub enum DirectoryItemType {
    SubDirectory,
    File,
    Symlink,
}

pub struct DirectoryItemDescriptor {
    pub item_type: DirectoryItemType,
    pub name: Option<String>,
    pub size: Option<u64>,
    pub last_modified: Option<io::Result<OffsetDateTime>>,
}

// Returns an HTML5 document describing the content of the given local directory.
pub fn build_html_directory_listing(summary: DirectorySummary) -> String {
    let mut page_html = String::with_capacity(1024);
    page_html.push_str(
        "<!DOCTYPE html>\
<html lang=\"en\">\
<head><title>Directory listing: ",
    );
    let directory_label = match summary.path {
        Ok(p) => p,
        Err(e) => format!("<{}>", e),
    };
    write_html_safe(&directory_label, &mut page_html);
    page_html.push_str("</title><style>");
    page_html.push_str(read_string(Resource::DirectoryListingCSS).as_str());
    page_html.push_str(
        "</style></head><body>\
<header><h1>Index of <span class=\"path\">",
    );
    write_html_safe(&directory_label, &mut page_html);
    page_html.push_str("</span></h1></header>");
    page_html.push_str("<div class=\"directory_info\">");
    if let Ok(items) = summary.items {
        let items_found = !&items.is_empty();
        if summary.has_parent {
            write_parent_link(&mut page_html);
        }
        if items_found {
            page_html.push_str("<div class=\"listing\">");
            for item in items {
                write_directory_listing_row(item.1, &mut page_html);
            }
            page_html.push_str("</div>");
        } else {
            page_html.push_str(
                "<div class=\"empty_notice\">\
<p>This directory is empty.</p></div>",
            );
        }
    } else {
        page_html.push_str("<p>Unable to list directory contents.</p>");
    }
    page_html.push_str(
        "</div><footer><p>Local directory listing generated by Servo.</p>\
</footer></body></html>",
    );
    page_html
}

fn write_directory_listing_row(descriptor: DirectoryItemDescriptor, page_html: &mut String) {
    page_html.push_str("<div class=\"row ");
    page_html.push_str(match descriptor.item_type {
        DirectoryItemType::SubDirectory => "directory",
        DirectoryItemType::File => "file",
        DirectoryItemType::Symlink => "symlink",
    });
    page_html.push_str("\">");
    match descriptor.item_type {
        DirectoryItemType::SubDirectory => write_directory_data(descriptor, page_html),
        _ => write_file_data(descriptor, page_html),
    }
    page_html.push_str("</div>");
}

fn write_parent_link(page_html: &mut String) {
    page_html.push_str("<div class=\"parent_link\">");
    page_html.push_str("<a href=\"../\">Up to parent directory</a>");
    page_html.push_str("</div>");
}

fn write_directory_data(descriptor: DirectoryItemDescriptor, page_html: &mut String) {
    page_html.push_str("<div class=\"name\">");
    if let Some(n) = descriptor.name {
        page_html.push_str("<a href=\"");
        write_html_safe(&n, page_html);
        page_html.push('/');
        page_html.push_str("\">");
        write_html_safe(&n, page_html);
        page_html.push('/');
        page_html.push_str("</a>");
    } else {
        page_html.push_str("&lt;invalid name&gt;");
    }
    page_html.push_str("</div><div class=\"size\">-</div>");
    page_html.push_str("<div class=\"modified\">");
    if let Some(Ok(last_mod)) = descriptor.last_modified {
        write_system_time(last_mod, page_html);
    }
    page_html.push_str("</div>");
}

fn write_file_data(descriptor: DirectoryItemDescriptor, page_html: &mut String) {
    page_html.push_str("<div class=\"name\">");
    if let Some(n) = descriptor.name {
        page_html.push_str("<a href=\"");
        write_html_safe(&n, page_html);
        page_html.push_str("\">");
        write_html_safe(&n, page_html);
        page_html.push_str("</a>");
    } else {
        page_html.push_str("&lt;invalid name&gt;");
    }
    page_html.push_str("</div><div class=\"size\">");
    if let Some(s) = descriptor.size {
        write_file_size(s, page_html);
    } else {
        page_html.push('-');
    }
    page_html.push_str("</div><div class=\"modified\">");
    if let Some(Ok(last_mod)) = descriptor.last_modified {
        write_system_time(last_mod, page_html);
    }
    page_html.push_str("</div>");
}

fn write_file_size(size: u64, page_html: &mut String) {
    if size < 1024 {
        page_html.push_str(size.to_string().as_str());
        page_html.push(' ');
        page_html.push_str("<abbr title=\"bytes\">B</abbr>");
    } else {
        let mut dec_size = size as f64;
        let mut prefix_power = 0;
        while dec_size > 1024.0 && prefix_power < 8 {
            dec_size /= 1024.0;
            prefix_power += 1;
        }
        let prefix = match prefix_power {
            1 => "<abbr title=\"kibibytes\">KiB</abbr>",
            2 => "<abbr title=\"mebibytes\">MiB</abbr>",
            3 => "<abbr title=\"gibibytes\">GiB</abbr>",
            4 => "<abbr title=\"tebibytes\">TiB</abbr>",
            5 => "<abbr title=\"pebibytes\">PiB</abbr>",
            6 => "<abbr title=\"exbibytes\">EiB</abbr>",
            7 => "<abbr title=\"zebibytes\">ZiB</abbr>",
            _ => "<abbr title=\"yobibytes\">YiB</abbr>",
        };
        page_html.push_str(format!("{:.2}", dec_size).as_str());
        page_html.push(' ');
        page_html.push_str(prefix);
    }
}

fn write_system_time(last_mod: OffsetDateTime, page_html: &mut String) {
    page_html.push_str("<time datetime=\"");
    write_datetime_iso_format(last_mod, page_html);
    page_html.push_str("\">");
    write_datetime_for_display(last_mod, page_html);
    page_html.push_str("</time>");
}

fn write_datetime_iso_format(last_mod: OffsetDateTime, page_html: &mut String) {
    page_html.push_str(format!("{:0>4}", last_mod.year()).as_str());
    page_html.push('-');
    let month_number: u8 = last_mod.month().into();
    page_html.push_str(format!("{:0>2}", month_number.to_string()).as_str());
    page_html.push('-');
    page_html.push_str(format!("{:0>2}", last_mod.day()).as_str());
    page_html.push('T');
    page_html.push_str(format!("{:0>2}", last_mod.hour()).as_str());
    page_html.push(':');
    page_html.push_str(format!("{:0>2}", last_mod.minute()).as_str());
    page_html.push(':');
    page_html.push_str(format!("{:0>2}", last_mod.second()).as_str());
    page_html.push('.');
    page_html.push_str(format!("{:0>3}", last_mod.millisecond()).as_str());
}

fn write_datetime_for_display(last_mod: OffsetDateTime, page_html: &mut String) {
    let now = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());
    page_html.push_str("<span class=\"date");
    if now.date().eq(&last_mod.date()) {
        page_html.push_str(" current");
    }
    page_html.push_str("\">");
    page_html.push_str(last_mod.day().to_string().as_str());
    page_html.push_str("<span class=\"day_ordinal_suffix\">");
    page_html.push_str(day_of_month_ordinal_suffix(last_mod.day()));
    page_html.push_str("</span> <span class=\"month\">");
    page_html.push_str(last_mod.month().to_string().as_str());
    page_html.push_str("</span> <span class=\"year");
    if last_mod.year() == now.year() {
        page_html.push_str(" current");
    }
    page_html.push_str("\">");
    page_html.push_str(last_mod.year().to_string().as_str());
    page_html.push_str("</span></span> ");

    page_html.push_str("<span class=\"time\">");
    page_html.push_str(format!("<span class=\"hour\">{:0>2}</span>", last_mod.hour()).as_str());
    page_html.push(':');
    page_html.push_str(format!("<span class=\"minute\">{:0>2}</span>", last_mod.minute()).as_str());
    page_html.push(':');
    page_html.push_str(format!("<span class=\"second\">{:0>2}</span>", last_mod.second()).as_str());
    page_html.push_str("</span>");
}

// Do not call this function with numbers outside the interval [1, 31].
pub fn day_of_month_ordinal_suffix(number: u8) -> &'static str {
    match number {
        1 | 21 | 31 => "st",
        2 | 22 => "nd",
        3 | 23 => "rd",
        _ => "th",
    }
}

/// Writes the given content to the given mutable String, escaping sensitive HTML characters.
///
/// Sensitive characters found within the given content will be replaced by HTML named character
/// references. The apostrophe and double-quote characters are also replaced, so that the content
/// can be safely written into an attribute within an HTML document.
///
/// # Examples
///
/// ```
/// let mut html_payload = String::new();
/// html_payload.push_str("<html><body><code class=\"example-html\">");
/// write_html_safe(
///     "<p class=\"demo\" id='1'>Alpha &amp; Omega</p>",
///     &mut html_payload,
/// );
/// html_payload.push_str("</code></body></html>");
/// assert_eq!(
///     &html_payload,
///     "<html><body><code class=\"example-html\">\
/// &lt;p class=&quot;demo&quot; id=&apos;1&apos;&gt;Alpha &amp;amp; Omega&lt;/p&gt;\
/// </code></body></html>"
/// );
/// ```
pub fn write_html_safe(content: &str, page_html: &mut String) {
    for c in content.chars() {
        match c {
            '<' => page_html.push_str("&lt;"),
            '&' => page_html.push_str("&amp;"),
            '>' => page_html.push_str("&gt;"),
            '\'' => page_html.push_str("&apos;"),
            '"' => page_html.push_str("&quot;"),
            _ => page_html.push(c),
        }
    }
}

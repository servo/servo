/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::BTreeMap;

use embedder_traits::resources::{read_string, Resource};
use net::local_directory_listing::{build_html_directory_listing, write_html_safe, DirectoryItem};
use time_03::format_description::well_known::Iso8601;
use time_03::OffsetDateTime;

enum FileType {
    File,
    Symlink,
}

enum BinarySizeUnit {
    Bytes,
    Kibibytes,
    Mebibytes,
    Gibibytes,
}

impl BinarySizeUnit {
    fn get_name(&self) -> &'static str {
        match self {
            Self::Bytes => "bytes",
            Self::Kibibytes => "kibibytes",
            Self::Mebibytes => "mebibytes",
            Self::Gibibytes => "gibibytes",
        }
    }

    fn get_suffix(&self) -> &'static str {
        match self {
            Self::Bytes => "B",
            Self::Kibibytes => "KiB",
            Self::Mebibytes => "MiB",
            Self::Gibibytes => "GiB",
        }
    }
}

#[test]
fn test_build_html_directory_listing_single_file() {
    let mut items = BTreeMap::new();
    items.insert(
        "README.txt".into(),
        create_file_descriptor(
            "README.txt",
            false,
            26,
            OffsetDateTime::parse("2023-01-01T00:00:00+00:00", &Iso8601::DEFAULT).unwrap(),
        ),
    );
    let (path, has_parent, items) = (
        Ok("/home/bob/demo/www/public_html/".to_string()),
        true,
        Ok(items),
    );
    let mut expected = String::with_capacity(1024);
    write_expected_start_and_header(&mut expected, "/home/bob/demo/www/public_html/", true);
    write_expected_file_row(
        &mut expected,
        "README.txt",
        FileType::File,
        "26",
        BinarySizeUnit::Bytes,
        OffsetDateTime::parse("2023-01-01T00:00:00+00:00", &Iso8601::DEFAULT).unwrap(),
    );
    write_expected_foooter_and_end(&mut expected);
    let result = build_html_directory_listing(path, has_parent, items);
    assert_eq!(result, expected);
}

#[test]
fn test_build_html_directory_listing_single_directory() {
    let mut items = BTreeMap::new();
    items.insert(
        "sub-directory".into(),
        create_directory_descriptor(
            "sub-directory",
            OffsetDateTime::parse("2023-12-31T23:59:59.999+00:00", &Iso8601::DEFAULT).unwrap(),
        ),
    );
    let (path, has_parent, items) = (Ok("/var/www/".to_string()), true, Ok(items));
    let mut expected = String::with_capacity(1024);
    write_expected_start_and_header(&mut expected, "/var/www/", true);
    write_expected_directory_row(
        &mut expected,
        "sub-directory",
        OffsetDateTime::parse("2023-12-31T23:59:59.999+00:00", &Iso8601::DEFAULT).unwrap(),
    );
    write_expected_foooter_and_end(&mut expected);
    let result = build_html_directory_listing(path, has_parent, items);
    assert_eq!(result, expected);
}

#[test]
fn test_build_html_directory_listing_root() {
    let mut items = BTreeMap::new();
    items.insert(
        "var".into(),
        create_directory_descriptor(
            "var",
            OffsetDateTime::parse("2023-02-17T22:31:21.386-05:00", &Iso8601::DEFAULT).unwrap(),
        ),
    );
    items.insert(
        "etc".into(),
        create_directory_descriptor(
            "etc",
            OffsetDateTime::parse("2023-02-17T22:31:21.387-05:00", &Iso8601::DEFAULT).unwrap(),
        ),
    );
    items.insert(
        "home".into(),
        create_directory_descriptor(
            "home",
            OffsetDateTime::parse("2023-02-17T22:31:34.212-05:00", &Iso8601::DEFAULT).unwrap(),
        ),
    );
    items.insert(
        ".hcwd".into(),
        create_file_descriptor(
            ".hcwd",
            false,
            0,
            OffsetDateTime::parse("2023-02-17T22:31:22.616-05:00", &Iso8601::DEFAULT).unwrap(),
        ),
    );
    let (path, has_parent, items) = (Ok("/".to_string()), false, Ok(items));
    let mut expected = String::with_capacity(1024);

    write_expected_start_and_header(&mut expected, "/", false);
    write_expected_file_row(
        &mut expected,
        ".hcwd",
        FileType::File,
        "0",
        BinarySizeUnit::Bytes,
        OffsetDateTime::parse("2023-02-17T22:31:22.616-05:00", &Iso8601::DEFAULT).unwrap(),
    );
    write_expected_directory_row(
        &mut expected,
        "etc",
        OffsetDateTime::parse("2023-02-17T22:31:21.387-05:00", &Iso8601::DEFAULT).unwrap(),
    );
    write_expected_directory_row(
        &mut expected,
        "home",
        OffsetDateTime::parse("2023-02-17T22:31:34.212-05:00", &Iso8601::DEFAULT).unwrap(),
    );
    write_expected_directory_row(
        &mut expected,
        "var",
        OffsetDateTime::parse("2023-02-17T22:31:21.386-05:00", &Iso8601::DEFAULT).unwrap(),
    );
    write_expected_foooter_and_end(&mut expected);
    let result = build_html_directory_listing(path, has_parent, items);
    assert_eq!(result, expected);
}

#[test]
fn test_build_html_directory_listing_home() {
    let mut items = BTreeMap::new();
    items.insert(
        "docs".into(),
        create_directory_descriptor(
            "docs",
            OffsetDateTime::parse("2024-06-20T20:41:21.999+01:00", &Iso8601::DEFAULT).unwrap(),
        ),
    );
    items.insert(
        ".profile".into(),
        create_file_descriptor(
            ".profile",
            false,
            21,
            OffsetDateTime::parse("2023-11-11T13:54:16.212+00:00", &Iso8601::DEFAULT).unwrap(),
        ),
    );
    items.insert(
        "minify-css.sh".into(),
        create_file_descriptor(
            "minify-css.sh",
            false,
            3_204,
            OffsetDateTime::parse("2022-12-01T18:34:08.321+00:00", &Iso8601::DEFAULT).unwrap(),
        ),
    );
    items.insert(
        "Lian and Jo 21st Party.wmv".into(),
        create_file_descriptor(
            "Lian and Jo 21st Party.wmv",
            false,
            115_731_474,
            OffsetDateTime::parse("2014-12-08T13:04:58.111+00:00", &Iso8601::DEFAULT).unwrap(),
        ),
    );
    items.insert(
        "ゴジラ - the proof.avi".into(),
        create_file_descriptor(
            "ゴジラ - the proof.avi",
            false,
            999_911_119_112,
            OffsetDateTime::parse("2024-05-01T12:00:00.000+01:00", &Iso8601::DEFAULT).unwrap(),
        ),
    );
    let (path, has_parent, items) = (Ok("/".to_string()), false, Ok(items));
    let mut expected = String::with_capacity(1024);

    write_expected_start_and_header(&mut expected, "/", false);
    write_expected_file_row(
        &mut expected,
        ".profile",
        FileType::File,
        "21",
        BinarySizeUnit::Bytes,
        OffsetDateTime::parse("2023-11-11T13:54:16.212+00:00", &Iso8601::DEFAULT).unwrap(),
    );
    write_expected_file_row(
        &mut expected,
        "Lian and Jo 21st Party.wmv",
        FileType::File,
        "110.37",
        BinarySizeUnit::Mebibytes,
        OffsetDateTime::parse("2014-12-08T13:04:58.111+00:00", &Iso8601::DEFAULT).unwrap(),
    );
    write_expected_directory_row(
        &mut expected,
        "docs",
        OffsetDateTime::parse("2024-06-20T20:41:21.999+01:00", &Iso8601::DEFAULT).unwrap(),
    );
    write_expected_file_row(
        &mut expected,
        "minify-css.sh",
        FileType::File,
        "3.13",
        BinarySizeUnit::Kibibytes,
        OffsetDateTime::parse("2022-12-01T18:34:08.321+00:00", &Iso8601::DEFAULT).unwrap(),
    );
    write_expected_file_row(
        &mut expected,
        "ゴジラ - the proof.avi",
        FileType::File,
        "931.24",
        BinarySizeUnit::Gibibytes,
        OffsetDateTime::parse("2024-05-01T12:00:00.000+01:00", &Iso8601::DEFAULT).unwrap(),
    );
    write_expected_foooter_and_end(&mut expected);
    let result = build_html_directory_listing(path, has_parent, items);
    assert_eq!(result, expected);
}

fn write_expected_start_and_header(expected: &mut String, directory_label: &str, has_parent: bool) {
    expected.push_str(
        "<!DOCTYPE html>\
<html lang=\"en\">\
<head><title>Directory listing: ",
    );
    expected.push_str(directory_label);
    expected.push_str("</title><style>");
    expected.push_str(read_string(Resource::DirectoryListingCSS).as_str());
    expected.push_str("</style></head><body>");
    expected.push_str("<header><h1>Index of <span class=\"path\">");
    expected.push_str(directory_label);
    expected.push_str(
        "</span>\
        </h1></header>",
    );
    expected.push_str("<div class=\"directory_info\">");
    if has_parent {
        expected.push_str(
            "<div class=\"parent_link\"><a href=\"../\">Up to parent directory</a></div>",
        );
    }
    expected.push_str("<div class=\"listing\">");
}

fn write_expected_file_row(
    expected: &mut String,
    filename: &str,
    is_symlink: FileType,
    filesize: &str,
    size_unit: BinarySizeUnit,
    last_modified: OffsetDateTime,
) {
    let row_type = match is_symlink {
        FileType::Symlink => "row symlink",
        _ => "row file",
    };
    let size_unit_name = size_unit.get_name();
    let size_unit_suffix = size_unit.get_suffix();
    expected.push_str(
        format!(
            "<div class=\"{row_type}\"><div class=\"name\">\
        <a href=\"{filename}\">{filename}</a>\
        </div><div class=\"size\">{filesize} \
        <abbr title=\"{size_unit_name}\">{size_unit_suffix}</abbr></div>\
        <div class=\"modified\">"
        )
        .as_str(),
    );
    expected.push_str("<time>");
    expected.push_str(
        last_modified
            .format(&Iso8601::DATE_TIME)
            .unwrap_or("Invalid datetime".to_string())
            .as_str(),
    );
    expected.push_str("</time></div></div>");
}

fn write_expected_directory_row(
    expected: &mut String,
    directory_name: &str,
    last_modified: OffsetDateTime,
) {
    expected.push_str(
        format!(
            "<div class=\"row directory\"><div class=\"name\">\
        <a href=\"{directory_name}/\">{directory_name}/</a>\
        </div><div class=\"size\">-</div>\
        <div class=\"modified\">"
        )
        .as_str(),
    );
    expected.push_str("<time>");
    expected.push_str(
        last_modified
            .format(&Iso8601::DATE_TIME)
            .unwrap_or("Invalid datetime".to_string())
            .as_str(),
    );
    expected.push_str("</time></div></div>");
}

fn write_expected_foooter_and_end(expected: &mut String) {
    expected.push_str(
        "</div></div>\
        <footer><p>Local directory listing generated by Servo.</p></footer></body></html>",
    );
}

fn create_file_descriptor(
    name: &str,
    symlink: bool,
    size: u64,
    last_modified: OffsetDateTime,
) -> DirectoryItem {
    DirectoryItem::File {
        is_symlink: symlink,
        name: {
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        },
        size,
        last_modified: Ok(last_modified),
    }
}

fn create_directory_descriptor(name: &str, last_modified: OffsetDateTime) -> DirectoryItem {
    DirectoryItem::SubDirectory {
        name: {
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        },
        last_modified: Ok(last_modified),
    }
}

#[test]
fn test_write_html_safe_empty_into_empty() {
    let mut s = String::new();
    write_html_safe("", &mut s);
    assert_eq!(&s, "");
}

#[test]
fn test_write_html_safe_insensitive_into_empty() {
    let mut s = String::new();
    write_html_safe("boring", &mut s);
    assert_eq!(&s, "boring");
}

#[test]
fn test_write_html_safe_sensitive_into_empty() {
    let mut s = String::new();
    write_html_safe(
        "<html><body><p class=\"demo\" id='1'>Words &amp; numbers</p></body></html>",
        &mut s,
    );
    assert_eq!(
        &s,
        "&lt;html&gt;&lt;body&gt;&lt;p class=&quot;demo&quot; \
id=&apos;1&apos;&gt;Words &amp;amp; numbers&lt;/p&gt;&lt;/body&gt;&lt;/html&gt;"
    );
}

#[test]
fn test_write_html_safe_empty_into_existing() {
    let mut s = String::new();
    s.push_str("silence:");
    write_html_safe("", &mut s);
    assert_eq!(&s, "silence:");
}

#[test]
fn test_write_html_safe_insensitive_into_existing() {
    let mut s = String::new();
    s.push_str("unit tests are never ");
    write_html_safe("boring", &mut s);
    assert_eq!(&s, "unit tests are never boring");
}

#[test]
fn test_write_html_safe_sensitive_into_existing() {
    let mut s = String::new();
    s.push_str("<html><body><code class=\"example-html\">");
    write_html_safe(
        "<html><body><p class=\"demo\" id='1'>Words &amp; numbers</p></body></html>",
        &mut s,
    );
    s.push_str("</code></body></html>");
    assert_eq!(
        &s,
        "<html><body><code class=\"example-html\">\
&lt;html&gt;&lt;body&gt;&lt;p class=&quot;demo&quot; \
id=&apos;1&apos;&gt;Words &amp;amp; numbers&lt;/p&gt;&lt;/body&gt;&lt;/html&gt;\
</code></body></html>"
    );
}

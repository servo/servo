/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::BTreeMap;

use embedder_traits::resources::{read_string, Resource};
use net::local_directory_listing::{
    build_html_directory_listing, day_of_month_ordinal_suffix, write_html_safe, DirectoryItem,
};
use time_03::{Date, Month, OffsetDateTime, Time};

enum FileType {
    File,
    Symlink,
}

enum DateType {
    NotToday,
    Today,
}

enum YearType {
    NotThisYear,
    CurrentYear,
}

enum BinarySizeUnit {
    Bytes,
    Kibibytes,
    Mebibytes,
    Gibibytes,
    Tebibytes,
    Pebibytes,
    Exbibytes,
    Zebibytes,
    Yobibytes,
}

impl BinarySizeUnit {
    fn get_name(&self) -> &'static str {
        match self {
            Self::Bytes => "bytes",
            Self::Kibibytes => "kibibytes",
            Self::Mebibytes => "mebibytes",
            Self::Gibibytes => "gibibytes",
            Self::Tebibytes => "tebibytes",
            Self::Pebibytes => "pebibytes",
            Self::Exbibytes => "exbibytes",
            Self::Zebibytes => "zebibytes",
            Self::Yobibytes => "yobibytes",
        }
    }

    fn get_suffix(&self) -> &'static str {
        match self {
            Self::Bytes => "B",
            Self::Kibibytes => "KiB",
            Self::Mebibytes => "MiB",
            Self::Gibibytes => "GiB",
            Self::Tebibytes => "TiB",
            Self::Pebibytes => "PiB",
            Self::Exbibytes => "EiB",
            Self::Zebibytes => "ZiB",
            Self::Yobibytes => "YiB",
        }
    }
}

#[test]
fn test_build_html_directory_listing_single_file() {
    let mut items = BTreeMap::new();
    items.insert(
        "README.txt".into(),
        create_file_descriptor("README.txt", false, 26, 2023, Month::January, 1, 0, 0, 0, 0),
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
        "2023-01-01T00:00:00.000",
        DateType::NotToday,
        "1",
        "st",
        "January",
        YearType::NotThisYear,
        "2023",
        "00",
        "00",
        "00",
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
        create_directory_descriptor("sub-directory", 2023, Month::December, 31, 23, 59, 59, 999),
    );
    let (path, has_parent, items) = (Ok("/var/www/".to_string()), true, Ok(items));
    let mut expected = String::with_capacity(1024);
    write_expected_start_and_header(&mut expected, "/var/www/", true);
    write_expected_directory_row(
        &mut expected,
        "sub-directory",
        "2023-12-31T23:59:59.999",
        DateType::NotToday,
        "31",
        "st",
        "December",
        YearType::NotThisYear,
        "2023",
        "23",
        "59",
        "59",
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
        create_directory_descriptor("var", 2023, Month::February, 17, 22, 31, 21, 386),
    );
    items.insert(
        "etc".into(),
        create_directory_descriptor("etc", 2023, Month::February, 17, 22, 31, 21, 387),
    );
    items.insert(
        "home".into(),
        create_directory_descriptor("home", 2023, Month::February, 17, 22, 31, 34, 212),
    );
    items.insert(
        ".hcwd".into(),
        create_file_descriptor(
            ".hcwd",
            false,
            0,
            2023,
            Month::February,
            17,
            22,
            31,
            22,
            616,
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
        "2023-02-17T22:31:22.616",
        DateType::NotToday,
        "17",
        "th",
        "February",
        YearType::NotThisYear,
        "2023",
        "22",
        "31",
        "22",
    );
    write_expected_directory_row(
        &mut expected,
        "etc",
        "2023-02-17T22:31:21.387",
        DateType::NotToday,
        "17",
        "th",
        "February",
        YearType::NotThisYear,
        "2023",
        "22",
        "31",
        "21",
    );
    write_expected_directory_row(
        &mut expected,
        "home",
        "2023-02-17T22:31:34.212",
        DateType::NotToday,
        "17",
        "th",
        "February",
        YearType::NotThisYear,
        "2023",
        "22",
        "31",
        "34",
    );
    write_expected_directory_row(
        &mut expected,
        "var",
        "2023-02-17T22:31:21.386",
        DateType::NotToday,
        "17",
        "th",
        "February",
        YearType::NotThisYear,
        "2023",
        "22",
        "31",
        "21",
    );
    write_expected_foooter_and_end(&mut expected);
    let result = build_html_directory_listing(path, has_parent, items);
    assert_eq!(result, expected);
}

#[test]
fn test_build_html_directory_listing_space_time_odyssey() {
    let now = OffsetDateTime::now_utc();
    let today = now.day();
    let not_today = match today {
        1 => 2,
        _ => today - 1,
    };
    let mut items = BTreeMap::new();
    items.insert(
        "archived".into(),
        create_directory_descriptor("archived", now.year(), now.month(), today, 12, 34, 56, 789),
    );
    items.insert(
        "rust-log".into(),
        create_file_descriptor(
            "rust-log",
            false,
            13,
            2023,
            now.month(),
            now.day(),
            22,
            31,
            22,
            333,
        ),
    );
    items.insert(
        "java-log".into(),
        create_file_descriptor(
            "java-log",
            false,
            88_128,
            now.year(),
            now.month(),
            not_today,
            6,
            30,
            28,
            734,
        ),
    );
    items.insert(
        "cobol-log".into(),
        create_file_descriptor(
            "cobol-log",
            false,
            18_963_218,
            1997,
            Month::March,
            21,
            2,
            7,
            52,
            298,
        ),
    );
    items.insert(
        "cpp-log".into(),
        create_file_descriptor(
            "cpp-log",
            false,
            566_242_762_437,
            now.year(),
            now.month(),
            today,
            5,
            22,
            6,
            992,
        ),
    );
    items.insert(
        "cs-log".into(),
        create_file_descriptor(
            "cs-log",
            false,
            93_477_133_853_546,
            now.year(),
            now.month(),
            today,
            23,
            17,
            0,
            515,
        ),
    );
    items.insert(
        "c-log".into(),
        create_file_descriptor(
            "c-log",
            false,
            988_193_477_133_853_546,
            now.year(),
            now.month(),
            today,
            23,
            57,
            58,
            119,
        ),
    );
    items.insert(
        "latest".into(),
        create_file_descriptor(
            "latest",
            true,
            19,
            now.year(),
            now.month(),
            today,
            23,
            57,
            58,
            120,
        ),
    );
    let (path, has_parent, items) = (Ok("/var/sys_logs/".to_string()), true, Ok(items));
    let now_year = now.year().to_string();
    let now_month_iso = format!("{:0>2}", u8::from(now.month()));
    let now_month_display = now.month().to_string();
    let now_day_iso = format!("{:0>2}", now.day());
    let now_day_display = now.day().to_string();
    let not_today_iso = format!("{:0>2}", not_today);
    let not_today_display = not_today.to_string();
    let today_suffix = day_of_month_ordinal_suffix(today);
    let not_today_suffix = day_of_month_ordinal_suffix(not_today);

    let mut expected = String::with_capacity(1024);
    write_expected_start_and_header(&mut expected, "/var/sys_logs/", true);
    write_expected_directory_row(
        &mut expected,
        "archived",
        format!("{now_year}-{now_month_iso}-{now_day_iso}T12:34:56.789",).as_str(),
        DateType::Today,
        &now_day_display,
        &today_suffix,
        &now_month_display,
        YearType::CurrentYear,
        &now_year.as_str(),
        "12",
        "34",
        "56",
    );
    write_expected_file_row(
        &mut expected,
        "c-log",
        FileType::File,
        "877.69",
        BinarySizeUnit::Pebibytes,
        format!("{now_year}-{now_month_iso}-{now_day_iso}T23:57:58.119").as_str(),
        DateType::Today,
        &now_day_display,
        &today_suffix,
        &now_month_display,
        YearType::CurrentYear,
        &now_year.as_str(),
        "23",
        "57",
        "58",
    );
    write_expected_file_row(
        &mut expected,
        "cobol-log",
        FileType::File,
        "18.08",
        BinarySizeUnit::Mebibytes,
        "1997-03-21T02:07:52.298",
        DateType::NotToday,
        "21",
        "st",
        "March",
        YearType::NotThisYear,
        "1997",
        "02",
        "07",
        "52",
    );
    write_expected_file_row(
        &mut expected,
        "cpp-log",
        FileType::File,
        "527.35",
        BinarySizeUnit::Gibibytes,
        format!("{now_year}-{now_month_iso}-{now_day_iso}T05:22:06.992").as_str(),
        DateType::Today,
        &now_day_display,
        &today_suffix,
        &now_month_display,
        YearType::CurrentYear,
        &now_year.as_str(),
        "05",
        "22",
        "06",
    );
    write_expected_file_row(
        &mut expected,
        "cs-log",
        FileType::File,
        "85.02",
        BinarySizeUnit::Tebibytes,
        format!("{now_year}-{now_month_iso}-{now_day_iso}T23:17:00.515").as_str(),
        DateType::Today,
        &now_day_display,
        &today_suffix,
        &now_month_display,
        YearType::CurrentYear,
        &now_year.as_str(),
        "23",
        "17",
        "00",
    );
    write_expected_file_row(
        &mut expected,
        "java-log",
        FileType::File,
        "86.06",
        BinarySizeUnit::Kibibytes,
        format!("{now_year}-{now_month_iso}-{not_today_iso}T06:30:28.734").as_str(),
        DateType::NotToday,
        &not_today_display,
        &not_today_suffix,
        &now_month_display,
        YearType::CurrentYear,
        &now_year.as_str(),
        "06",
        "30",
        "28",
    );
    write_expected_file_row(
        &mut expected,
        "latest",
        FileType::Symlink,
        "19",
        BinarySizeUnit::Bytes,
        format!("{now_year}-{now_month_iso}-{now_day_iso}T23:57:58.120").as_str(),
        DateType::Today,
        &now_day_display,
        &today_suffix,
        &now_month_display,
        YearType::CurrentYear,
        &now_year.as_str(),
        "23",
        "57",
        "58",
    );
    write_expected_file_row(
        &mut expected,
        "rust-log",
        FileType::File,
        "13",
        BinarySizeUnit::Bytes,
        format!("2023-{now_month_iso}-{now_day_iso}T22:31:22.333").as_str(),
        DateType::NotToday,
        &now_day_display,
        &today_suffix,
        &now_month_display,
        YearType::NotThisYear,
        "2023",
        "22",
        "31",
        "22",
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
    iso_datetime: &str,
    day_is_current: DateType,
    now_day_display: &str,
    today_suffix: &str,
    now_month_display: &str,
    year_is_current: YearType,
    now_year_display: &str,
    now_hour_display: &str,
    now_minute_display: &str,
    now_second_display: &str,
) {
    let row_type = match is_symlink {
        FileType::Symlink => "row symlink",
        _ => "row file",
    };
    let day_current = match day_is_current {
        DateType::Today => " current",
        _ => "",
    };
    let year_current = match year_is_current {
        YearType::CurrentYear => " current",
        _ => "",
    };
    let size_unit_name = size_unit.get_name();
    let size_unit_suffix = size_unit.get_suffix();
    expected.push_str(
        format!(
            "<div class=\"{row_type}\"><div class=\"name\">\
        <a href=\"{filename}\">{filename}</a>\
        </div><div class=\"size\">{filesize} \
        <abbr title=\"{size_unit_name}\">{size_unit_suffix}</abbr></div>\
        <div class=\"modified\">\
        <time datetime=\"{iso_datetime}\">\
        <span class=\"date{day_current}\">{now_day_display}\
        <span class=\"day_ordinal_suffix\">{today_suffix}</span> \
        <span class=\"month\">{now_month_display}</span> \
        <span class=\"year{year_current}\">{now_year_display}</span></span> \
        <span class=\"time\"><span class=\"hour\">{now_hour_display}</span>:\
        <span class=\"minute\">{now_minute_display}</span>:\
        <span class=\"second\">{now_second_display}</span></span></time></div></div>",
        )
        .as_str(),
    );
}

fn write_expected_directory_row(
    expected: &mut String,
    directory_name: &str,
    iso_datetime: &str,
    day_type: DateType,
    now_day_display: &str,
    today_suffix: &str,
    now_month_display: &str,
    year_type: YearType,
    now_year_display: &str,
    now_hour_display: &str,
    now_minute_display: &str,
    now_second_display: &str,
) {
    let day_current = match day_type {
        DateType::Today => " current",
        _ => "",
    };
    let year_current = match year_type {
        YearType::CurrentYear => " current",
        _ => "",
    };
    expected.push_str(
        format!(
            "<div class=\"row directory\"><div class=\"name\">\
        <a href=\"{directory_name}/\">{directory_name}/</a>\
        </div><div class=\"size\">-</div>\
        <div class=\"modified\">\
        <time datetime=\"{iso_datetime}\">\
        <span class=\"date{day_current}\">{now_day_display}\
        <span class=\"day_ordinal_suffix\">{today_suffix}</span> \
        <span class=\"month\">{now_month_display}</span> \
        <span class=\"year{year_current}\">{now_year_display}</span></span> \
        <span class=\"time\"><span class=\"hour\">{now_hour_display}</span>:\
        <span class=\"minute\">{now_minute_display}</span>:\
        <span class=\"second\">{now_second_display}</span></span></time></div></div>"
        )
        .as_str(),
    );
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
    mod_year: i32,
    mod_month: Month,
    mod_day: u8,
    mod_hour: u8,
    mod_minute: u8,
    mod_second: u8,
    mod_milli: u16,
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
        last_modified: Ok(OffsetDateTime::new_utc(
            Date::from_calendar_date(mod_year, mod_month, mod_day).unwrap(),
            Time::from_hms_milli(mod_hour, mod_minute, mod_second, mod_milli).unwrap(),
        )),
    }
}

fn create_directory_descriptor(
    name: &str,
    mod_year: i32,
    mod_month: Month,
    mod_day: u8,
    mod_hour: u8,
    mod_minute: u8,
    mod_second: u8,
    mod_milli: u16,
) -> DirectoryItem {
    DirectoryItem::SubDirectory {
        name: {
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        },
        last_modified: Ok(OffsetDateTime::new_utc(
            Date::from_calendar_date(mod_year, mod_month, mod_day).unwrap(),
            Time::from_hms_milli(mod_hour, mod_minute, mod_second, mod_milli).unwrap(),
        )),
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

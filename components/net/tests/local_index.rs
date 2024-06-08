#![cfg(not(target_os = "windows"))]

use std::collections::BTreeMap;
use std::fs::{create_dir, File};
use std::sync::Arc;

use embedder_traits::resources::{read_string, Resource};
use headers::{ContentType, HeaderMapExt};
use mime::{self, Mime};
use net::local_directory_listing::{
    build_directory_summary, build_html_directory_listing, day_of_month_ordinal_suffix,
    write_html_safe, DirectoryItemDescriptor, DirectoryItemType, DirectorySummary,
};
use net::resource_thread::CoreResourceThreadPool;
use net_traits::request::{Origin, Referrer, Request};
use net_traits::response::{HttpsState, ResponseBody, ResponseType};
use net_traits::NetworkError;
use servo_config::pref;
use servo_url::ServoUrl;
use tempfile::{tempdir, TempDir};
use time_03::{Date, Month, OffsetDateTime, Time};

use crate::{fetch_with_context, new_fetch_context, Path};

#[test]
fn test_local_directory_listing_forbidden() {
    if pref!(network.local_directory_listing.enabled) {
        // Wrong preference setting for this unit test, so report success and skip actual code.
        return;
    }
    let path = Path::new("../../tests/").canonicalize().unwrap();
    let url = ServoUrl::from_file_path(path.clone()).unwrap();

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(
        url,
        Some(origin),
        Referrer::NoReferrer,
        None,
        HttpsState::None,
    );

    let pool = CoreResourceThreadPool::new(1);
    let pool_handle = Arc::new(pool);
    let mut context = new_fetch_context(None, None, Some(Arc::downgrade(&pool_handle)));
    let fetch_response = fetch_with_context(&mut request, &mut context);

    // With the "local_directory_listing" preference disabled, we
    // should receive an error if we try to open a local directory
    // path.
    assert_eq!(
        fetch_response.response_type,
        ResponseType::Error(NetworkError::Internal(
            "Local directory listing feature has not been enabled in preferences".into()
        ))
    );
}

#[test]
fn test_local_directory_listing_tempdir() {
    if !pref!(network.local_directory_listing.enabled) {
        // Wrong preference setting for this unit test, so report success and skip actual code.
        return;
    }
    let mut temp_dir = tempdir().unwrap();
    /*
    Do not allow unwrap/expect/panic from now on, until the temporary directory is closed.
    Otherwise, there's a chance the temporary directory will not be deleted.
    */
    let url = {
        if populate_temporary_directory(&mut temp_dir).is_err() {
            if temp_dir.close().is_err() {
                panic!("tempfile failed to close temporary directory!");
            }
            panic!("Failed to populate temporary directory with test data!");
        }
        if let Ok(path) = Path::new(&temp_dir.path()).canonicalize() {
            if let Ok(servo_url) = ServoUrl::from_file_path(path.clone()) {
                servo_url
            } else {
                if temp_dir.close().is_err() {
                    panic!("tempfile failed to close temporary directory!");
                }
                panic!("Failed to create ServoUrl for temporary directory!");
            }
        } else {
            if temp_dir.close().is_err() {
                panic!("tempfile failed to close temporary directory!");
            }
            panic!("Failed to canonicalize path to temporary directory!");
        }
    };

    let origin = Origin::Origin(url.origin());
    let mut request = Request::new(
        url,
        Some(origin),
        Referrer::NoReferrer,
        None,
        HttpsState::None,
    );

    let pool = CoreResourceThreadPool::new(1);
    let pool_handle = Arc::new(pool);
    let mut context = new_fetch_context(None, None, Some(Arc::downgrade(&pool_handle)));
    let fetch_response = fetch_with_context(&mut request, &mut context);

    let temp_path = temp_dir.path().to_str().unwrap_or("").to_string();
    let temp_path = temp_path.as_str();
    if temp_dir.close().is_err() {
        panic!("tempfile failed to close temporary directory!");
    }

    // We should see an opaque-filtered response.
    assert_eq!(fetch_response.response_type, ResponseType::Opaque);

    assert!(!fetch_response.is_network_error());
    assert_eq!(fetch_response.headers.len(), 0);
    let resp_body = fetch_response.body.lock().unwrap();
    assert_eq!(*resp_body, ResponseBody::Empty);

    // The underlying response behind the filter should
    // have the file's MIME type and contents.
    let actual_response = fetch_response.actual_response();
    assert!(!actual_response.is_network_error());
    assert_eq!(actual_response.headers.len(), 1);
    let content_type: Mime = actual_response
        .headers
        .typed_get::<ContentType>()
        .unwrap()
        .into();
    assert_eq!(content_type, mime::TEXT_HTML);

    let resp_body = actual_response.body.lock().unwrap();

    match *resp_body {
        ResponseBody::Done(ref val) => {
            let string_val = String::from_utf8(val.to_vec()).unwrap();
            assert!(string_val.starts_with("<!DOCTYPE html>"));
            assert!(string_val
                .contains(format!("<title>Directory listing: {temp_path}</title>").as_str()));
            assert!(string_val.contains(
                format!("<h1>Index of <span class=\"path\">{temp_path}</span></h1>").as_str()
            ));
            assert!(string_val.contains(
                "<div class=\"directory_info\">\
<div class=\"parent_link\"><a href=\"../\">Up to parent directory</a></div>\
<div class=\"listing\">"
            ));
            assert!(string_val.contains(
                "<div class=\"row directory\"><div class=\"name\">\
<a href=\"html&lt;unsafe&amp;weird&gt;directoryname/\">\
html&lt;unsafe&amp;weird&gt;directoryname/</a></div>\
<div class=\"size\">-</div>\
<div class=\"modified\"><time datetime=\""
            ));
            assert!(string_val.contains(
                "<div class=\"row directory\"><div class=\"name\">\
<a href=\"sub-directory-example/\">\
sub-directory-example/</a></div>\
<div class=\"size\">-</div>\
<div class=\"modified\"><time datetime=\""
            ));
            assert!(string_val.contains(
                "<div class=\"row file\"><div class=\"name\">\
<a href=\"html&lt;unsafe&amp;weird&gt;filename.html\">\
html&lt;unsafe&amp;weird&gt;filename.html</a></div>\
<div class=\"size\">88 <abbr title=\"bytes\">B</abbr></div>\
<div class=\"modified\"><time datetime=\""
            ));
            assert!(string_val.contains(
                "<div class=\"row file\"><div class=\"name\">\
<a href=\"README.txt\">\
README.txt</a></div>\
<div class=\"size\">375 <abbr title=\"bytes\">B</abbr></div>\
<div class=\"modified\"><time datetime=\""
            ));
            assert!(string_val.contains(
                "<div class=\"row file\"><div class=\"name\">\
<a href=\"some_file_name.html\">\
some_file_name.html</a></div>\
<div class=\"size\">102 <abbr title=\"bytes\">B</abbr></div>\
<div class=\"modified\"><time datetime=\""
            ));
        },
        _ => panic!(),
    }
}

#[test]
fn test_build_directory_summary() {
    let mut temp_dir = tempdir().unwrap();
    let completed_successfully = populate_temporary_directory(&mut temp_dir).is_ok();
    if temp_dir.close().is_err() {
        panic!("tempfile failed to close temporary directory!");
    }
    assert!(completed_successfully);
}

// NOTE: THIS FUNCTION IS RELIED UPON BY MORE THAN ONE UNIT TEST - BE WARY OF MAKING CHANGES !!!
fn populate_temporary_directory(temp_dir: &mut TempDir) -> std::io::Result<()> {
    // Do not use unsafe/panic within this block, otherwise temp_dir.close() may not be called!
    create_dir(temp_dir.path().join("html<unsafe&weird>directoryname"))?;
    create_dir(temp_dir.path().join("sub-directory-example"))?;
    let file = File::create(temp_dir.path().join("html<unsafe&weird>filename.html"))?;
    file.set_len(88).unwrap();
    let file = File::create(temp_dir.path().join("some_file_name.html"))?;
    file.set_len(102).unwrap();
    let file = File::create(temp_dir.path().join("README.txt"))?;
    file.set_len(375).unwrap();
    // TODO: WORK OUT HOW TO ADD A SYMLINK (platform-dependent) !!!
    let summary = build_directory_summary(temp_dir.path().to_path_buf());
    assert_eq!(summary.items.as_ref().map(|m| m.len()).unwrap_or(0), 5);
    assert!(summary_contains_file(&summary, "README.txt", 375));
    assert!(summary_contains_file(
        &summary,
        "html<unsafe&weird>filename.html",
        88
    ));
    assert!(summary_contains_file(&summary, "some_file_name.html", 102));
    assert!(summary_contains_directory(
        &summary,
        "html<unsafe&weird>directoryname"
    ));
    assert!(summary_contains_directory(
        &summary,
        "sub-directory-example"
    ));
    Ok(())
}

fn summary_contains_file(summary: &DirectorySummary, name: &str, size: u64) -> bool {
    if let Ok(items) = summary.items.as_ref() {
        for item in items {
            let descriptor = item.1;
            if !matches!(descriptor.item_type, DirectoryItemType::File) {
                continue;
            }
            if let Some(d_name) = &descriptor.name {
                if d_name == name {
                    if let Some(d_size) = &descriptor.size {
                        if d_size == &size {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

fn summary_contains_directory(summary: &DirectorySummary, name: &str) -> bool {
    if let Ok(items) = summary.items.as_ref() {
        for item in items {
            let descriptor = item.1;
            if !matches!(descriptor.item_type, DirectoryItemType::SubDirectory) {
                continue;
            }
            if let Some(d_name) = &descriptor.name {
                if d_name == name {
                    return true;
                }
            }
        }
    }
    false
}

#[test]
fn test_build_html_directory_listing_single_file() {
    let mut items = BTreeMap::new();
    items.insert(
        "README.txt".into(),
        create_file_descriptor("README.txt", false, 26, 2023, Month::January, 1, 0, 0, 0, 0),
    );
    let summary = DirectorySummary {
        path: Ok("/home/bob/demo/www/public_html/".to_string()),
        has_parent: true,
        items: Ok(items),
    };
    let mut expected = String::with_capacity(1024);
    expected.push_str(
        "<!DOCTYPE html>\
<html lang=\"en\">\
<head><title>Directory listing: /home/bob/demo/www/public_html/</title><style>",
    );
    expected.push_str(read_string(Resource::DirectoryListingCSS).as_str());
    expected.push_str("</style></head><body>");
    expected.push_str(
        "<header><h1>Index of <span class=\"path\">/home/bob/demo/www/public_html/</span>\
        </h1></header>",
    );
    expected.push_str(
        "<div class=\"directory_info\"><div class=\"parent_link\">\
        <a href=\"../\">Up to parent directory</a></div><div class=\"listing\">\
        <div class=\"row file\"><div class=\"name\"><a href=\"README.txt\">README.txt</a>\
        </div><div class=\"size\">26 <abbr title=\"bytes\">B</abbr></div>\
        <div class=\"modified\"><time datetime=\"2023-01-01T00:00:00.000\">\
        <span class=\"date\">1<span class=\"day_ordinal_suffix\">st</span> \
        <span class=\"month\">January</span> <span class=\"year\">2023</span></span> \
        <span class=\"time\"><span class=\"hour\">00</span>:<span class=\"minute\">00</span>:\
        <span class=\"second\">00</span></span></time></div></div></div></div>\
        <footer><p>Local directory listing generated by Servo.</p></footer></body></html>",
    );
    let result = build_html_directory_listing(summary);
    assert_eq!(result, expected);
}

#[test]
fn test_build_html_directory_listing_single_directory() {
    let mut items = BTreeMap::new();
    items.insert(
        "sub-directory".into(),
        create_directory_descriptor("sub-directory", 2023, Month::December, 31, 23, 59, 59, 999),
    );
    let summary = DirectorySummary {
        path: Ok("/var/www/".to_string()),
        has_parent: true,
        items: Ok(items),
    };
    let mut expected = String::with_capacity(1024);
    expected.push_str(
        "<!DOCTYPE html>\
<html lang=\"en\">\
<head><title>Directory listing: /var/www/</title><style>",
    );
    expected.push_str(read_string(Resource::DirectoryListingCSS).as_str());
    expected.push_str("</style></head><body>");
    expected.push_str(
        "<header><h1>Index of <span class=\"path\">/var/www/</span>\
        </h1></header>",
    );
    expected.push_str(
        "<div class=\"directory_info\"><div class=\"parent_link\">\
        <a href=\"../\">Up to parent directory</a></div><div class=\"listing\">\
        <div class=\"row directory\"><div class=\"name\">\
        <a href=\"sub-directory/\">sub-directory/</a>\
        </div><div class=\"size\">-</div>\
        <div class=\"modified\"><time datetime=\"2023-12-31T23:59:59.999\">\
        <span class=\"date\">31<span class=\"day_ordinal_suffix\">st</span> \
        <span class=\"month\">December</span> <span class=\"year\">2023</span></span> \
        <span class=\"time\"><span class=\"hour\">23</span>:<span class=\"minute\">59</span>:\
        <span class=\"second\">59</span></span></time></div></div></div></div>\
        <footer><p>Local directory listing generated by Servo.</p></footer></body></html>",
    );
    let result = build_html_directory_listing(summary);
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
    let summary = DirectorySummary {
        path: Ok("/".to_string()),
        has_parent: false,
        items: Ok(items),
    };
    let mut expected = String::with_capacity(1024);
    expected.push_str(
        "<!DOCTYPE html>\
<html lang=\"en\">\
<head><title>Directory listing: /</title><style>",
    );
    expected.push_str(read_string(Resource::DirectoryListingCSS).as_str());
    expected.push_str("</style></head><body>");
    expected.push_str(
        "<header><h1>Index of <span class=\"path\">/</span>\
        </h1></header>",
    );
    expected.push_str(
        "<div class=\"directory_info\"><div class=\"listing\">\
        <div class=\"row file\"><div class=\"name\">\
        <a href=\".hcwd\">.hcwd</a>\
        </div><div class=\"size\">0 <abbr title=\"bytes\">B</abbr></div>\
        <div class=\"modified\"><time datetime=\"2023-02-17T22:31:22.616\">\
        <span class=\"date\">17<span class=\"day_ordinal_suffix\">th</span> \
        <span class=\"month\">February</span> <span class=\"year\">2023</span></span> \
        <span class=\"time\"><span class=\"hour\">22</span>:<span class=\"minute\">31</span>:\
        <span class=\"second\">22</span></span></time></div></div>\
        <div class=\"row directory\"><div class=\"name\">\
        <a href=\"etc/\">etc/</a>\
        </div><div class=\"size\">-</div>\
        <div class=\"modified\"><time datetime=\"2023-02-17T22:31:21.387\">\
        <span class=\"date\">17<span class=\"day_ordinal_suffix\">th</span> \
        <span class=\"month\">February</span> <span class=\"year\">2023</span></span> \
        <span class=\"time\"><span class=\"hour\">22</span>:<span class=\"minute\">31</span>:\
        <span class=\"second\">21</span></span></time></div></div>\
        <div class=\"row directory\"><div class=\"name\">\
        <a href=\"home/\">home/</a>\
        </div><div class=\"size\">-</div>\
        <div class=\"modified\"><time datetime=\"2023-02-17T22:31:34.212\">\
        <span class=\"date\">17<span class=\"day_ordinal_suffix\">th</span> \
        <span class=\"month\">February</span> <span class=\"year\">2023</span></span> \
        <span class=\"time\"><span class=\"hour\">22</span>:<span class=\"minute\">31</span>:\
        <span class=\"second\">34</span></span></time></div></div>\
        <div class=\"row directory\"><div class=\"name\">\
        <a href=\"var/\">var/</a>\
        </div><div class=\"size\">-</div>\
        <div class=\"modified\"><time datetime=\"2023-02-17T22:31:21.386\">\
        <span class=\"date\">17<span class=\"day_ordinal_suffix\">th</span> \
        <span class=\"month\">February</span> <span class=\"year\">2023</span></span> \
        <span class=\"time\"><span class=\"hour\">22</span>:<span class=\"minute\">31</span>:\
        <span class=\"second\">21</span></span></time></div></div>\
        </div></div>\
        <footer><p>Local directory listing generated by Servo.</p></footer></body></html>",
    );
    let result = build_html_directory_listing(summary);
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
    let summary = DirectorySummary {
        path: Ok("/var/sys_logs/".to_string()),
        has_parent: true,
        items: Ok(items),
    };
    let mut expected = String::with_capacity(1024);
    expected.push_str(
        "<!DOCTYPE html>\
<html lang=\"en\">\
<head><title>Directory listing: /var/sys_logs/</title><style>",
    );
    expected.push_str(read_string(Resource::DirectoryListingCSS).as_str());
    expected.push_str("</style></head><body>");
    expected.push_str(
        "<header><h1>Index of <span class=\"path\">/var/sys_logs/</span>\
        </h1></header>",
    );
    let now_year = now.year().to_string();
    let now_month_iso = format!("{:0>2}", u8::from(now.month()));
    let now_month_display = now.month().to_string();
    let now_day_iso = format!("{:0>2}", now.day());
    let now_day_display = now.day().to_string();
    let not_today_iso = format!("{:0>2}", not_today);
    let not_today_display = not_today.to_string();
    let today_suffix = day_of_month_ordinal_suffix(today);
    let not_today_suffix = day_of_month_ordinal_suffix(not_today);
    expected.push_str(
        format!(
            "<div class=\"directory_info\"><div class=\"parent_link\">\
        <a href=\"../\">Up to parent directory</a></div><div class=\"listing\">\
        <div class=\"row directory\"><div class=\"name\">\
        <a href=\"archived/\">archived/</a>\
        </div><div class=\"size\">-</div>\
        <div class=\"modified\">\
        <time datetime=\"{now_year}-{now_month_iso}-{now_day_iso}T12:34:56.789\">\
        <span class=\"date current\">{now_day_display}\
        <span class=\"day_ordinal_suffix\">{today_suffix}</span> \
        <span class=\"month\">{now_month_display}</span> \
        <span class=\"year current\">{now_year}</span></span> \
        <span class=\"time\"><span class=\"hour\">12</span>:<span class=\"minute\">34</span>:\
        <span class=\"second\">56</span></span></time></div></div>\
        <div class=\"row file\"><div class=\"name\">\
        <a href=\"c-log\">c-log</a>\
        </div><div class=\"size\">877.69 <abbr title=\"pebibytes\">PiB</abbr></div>\
        <div class=\"modified\">\
        <time datetime=\"{now_year}-{now_month_iso}-{now_day_iso}T23:57:58.119\">\
        <span class=\"date current\">{now_day_display}\
        <span class=\"day_ordinal_suffix\">{today_suffix}</span> \
        <span class=\"month\">{now_month_display}</span> \
        <span class=\"year current\">{now_year}</span></span> \
        <span class=\"time\"><span class=\"hour\">23</span>:<span class=\"minute\">57</span>:\
        <span class=\"second\">58</span></span></time></div></div>\
        <div class=\"row file\"><div class=\"name\">\
        <a href=\"cobol-log\">cobol-log</a>\
        </div><div class=\"size\">18.08 <abbr title=\"mebibytes\">MiB</abbr></div>\
        <div class=\"modified\"><time datetime=\"1997-03-21T02:07:52.298\">\
        <span class=\"date\">21<span class=\"day_ordinal_suffix\">st</span> \
        <span class=\"month\">March</span> <span class=\"year\">1997</span></span> \
        <span class=\"time\"><span class=\"hour\">02</span>:<span class=\"minute\">07</span>:\
        <span class=\"second\">52</span></span></time></div></div>\
        <div class=\"row file\"><div class=\"name\">\
        <a href=\"cpp-log\">cpp-log</a>\
        </div><div class=\"size\">527.35 <abbr title=\"gibibytes\">GiB</abbr></div>\
        <div class=\"modified\">\
        <time datetime=\"{now_year}-{now_month_iso}-{now_day_iso}T05:22:06.992\">\
        <span class=\"date current\">{now_day_display}\
        <span class=\"day_ordinal_suffix\">{today_suffix}</span> \
        <span class=\"month\">{now_month_display}</span> \
        <span class=\"year current\">{now_year}</span></span> \
        <span class=\"time\"><span class=\"hour\">05</span>:<span class=\"minute\">22</span>:\
        <span class=\"second\">06</span></span></time></div></div>\
        <div class=\"row file\"><div class=\"name\">\
        <a href=\"cs-log\">cs-log</a>\
        </div><div class=\"size\">85.02 <abbr title=\"tebibytes\">TiB</abbr></div>\
        <div class=\"modified\">\
        <time datetime=\"{now_year}-{now_month_iso}-{now_day_iso}T23:17:00.515\">\
        <span class=\"date current\">{now_day_display}\
        <span class=\"day_ordinal_suffix\">{today_suffix}</span> \
        <span class=\"month\">{now_month_display}</span> \
        <span class=\"year current\">{now_year}</span></span> \
        <span class=\"time\"><span class=\"hour\">23</span>:<span class=\"minute\">17</span>:\
        <span class=\"second\">00</span></span></time></div></div>\
        <div class=\"row file\"><div class=\"name\">\
        <a href=\"java-log\">java-log</a>\
        </div><div class=\"size\">86.06 <abbr title=\"kibibytes\">KiB</abbr></div>\
        <div class=\"modified\">\
        <time datetime=\"{now_year}-{now_month_iso}-{not_today_iso}T06:30:28.734\">\
        <span class=\"date\">{not_today_display}\
        <span class=\"day_ordinal_suffix\">{not_today_suffix}</span> \
        <span class=\"month\">{now_month_display}</span> \
        <span class=\"year current\">{now_year}</span></span> \
        <span class=\"time\"><span class=\"hour\">06</span>:<span class=\"minute\">30</span>:\
        <span class=\"second\">28</span></span></time></div></div>\
        <div class=\"row symlink\"><div class=\"name\">\
        <a href=\"latest\">latest</a>\
        </div><div class=\"size\">19 <abbr title=\"bytes\">B</abbr></div>\
        <div class=\"modified\">\
        <time datetime=\"{now_year}-{now_month_iso}-{now_day_iso}T23:57:58.120\">\
        <span class=\"date current\">{now_day_display}\
        <span class=\"day_ordinal_suffix\">{today_suffix}</span> \
        <span class=\"month\">{now_month_display}</span> \
        <span class=\"year current\">{now_year}</span></span> \
        <span class=\"time\"><span class=\"hour\">23</span>:<span class=\"minute\">57</span>:\
        <span class=\"second\">58</span></span></time></div></div>\
        <div class=\"row file\"><div class=\"name\">\
        <a href=\"rust-log\">rust-log</a>\
        </div><div class=\"size\">13 <abbr title=\"bytes\">B</abbr></div>\
        <div class=\"modified\">\
        <time datetime=\"2023-{now_month_iso}-{now_day_iso}T22:31:22.333\">\
        <span class=\"date\">{now_day_display}\
        <span class=\"day_ordinal_suffix\">{today_suffix}</span> \
        <span class=\"month\">{now_month_display}</span> \
        <span class=\"year\">2023</span></span> \
        <span class=\"time\"><span class=\"hour\">22</span>:<span class=\"minute\">31</span>:\
        <span class=\"second\">22</span></span></time></div></div>\
        </div></div>\
        <footer><p>Local directory listing generated by Servo.</p></footer></body></html>",
        )
        .as_str(),
    );
    let result = build_html_directory_listing(summary);
    assert_eq!(result, expected);
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
) -> DirectoryItemDescriptor {
    DirectoryItemDescriptor {
        item_type: match symlink {
            true => DirectoryItemType::Symlink,
            false => DirectoryItemType::File,
        },
        name: if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        },
        size: Some(size),
        last_modified: Some(Ok(OffsetDateTime::new_utc(
            Date::from_calendar_date(mod_year, mod_month, mod_day).unwrap(),
            Time::from_hms_milli(mod_hour, mod_minute, mod_second, mod_milli).unwrap(),
        ))),
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
) -> DirectoryItemDescriptor {
    DirectoryItemDescriptor {
        item_type: DirectoryItemType::SubDirectory,
        name: if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        },
        size: None,
        last_modified: Some(Ok(OffsetDateTime::new_utc(
            Date::from_calendar_date(mod_year, mod_month, mod_day).unwrap(),
            Time::from_hms_milli(mod_hour, mod_minute, mod_second, mod_milli).unwrap(),
        ))),
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

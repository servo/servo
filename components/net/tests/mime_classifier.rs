/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::path::{self, PathBuf};

use mime::{self, Mime};
use net::mime_classifier::{ApacheBugFlag, MimeClassifier, Mp4Matcher, NoSniffFlag};
use net_traits::LoadContext;

fn read_file(path: &path::Path) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

#[test]
fn test_sniff_mp4_matcher() {
    let matcher = Mp4Matcher;

    let p = PathBuf::from("tests/parsable_mime/video/mp4/test.mp4");
    let read_result = read_file(&p);

    match read_result {
        Ok(data) => {
            println!("Data Length {:?}", data.len());
            if !matcher.matches(&data) {
                panic!("Didn't read mime type")
            }
        },
        Err(e) => panic!("Couldn't read from file with error {}", e),
    }
}

#[test]
fn test_sniff_mp4_matcher_long() {
    // Check that a multi-byte length is calculated correctly
    let matcher = Mp4Matcher;

    let mut data: [u8; 260] = [0; 260];
    let _ = &data[..11].clone_from_slice(&[
        0x00, 0x00, 0x01, 0x04, 0x66, 0x74, 0x79, 0x70, 0x6D, 0x70, 0x34,
    ]);

    assert!(matcher.matches(&data));
}

#[test]
fn test_validate_classifier() {
    let classifier = MimeClassifier::default();
    classifier.validate().expect("Validation error")
}

#[cfg(test)]
fn test_sniff_with_flags(
    filename_orig: &path::Path,
    expected_mime: Mime,
    supplied_type: Option<Mime>,
    no_sniff_flag: NoSniffFlag,
    apache_bug_flag: ApacheBugFlag,
) {
    let current_working_directory = env::current_dir().unwrap();
    println!(
        "The current directory is {}",
        current_working_directory.display()
    );

    let mut filename = PathBuf::from("tests/parsable_mime/");
    filename.push(filename_orig);

    let classifier = MimeClassifier::default();

    let read_result = read_file(&filename);

    match read_result {
        Ok(data) => {
            let parsed_mime = classifier.classify(
                LoadContext::Browsing,
                no_sniff_flag,
                apache_bug_flag,
                &supplied_type,
                &data,
            );
            if (parsed_mime.type_() != expected_mime.type_()) ||
                (parsed_mime.subtype() != expected_mime.subtype())
            {
                panic!(
                    "File {:?} parsed incorrectly should be {:?}, parsed as {:?}",
                    filename, expected_mime, parsed_mime
                );
            }
        },
        Err(e) => panic!("Couldn't read from file {:?} with error {}", filename, e),
    }
}

#[cfg(test)]
fn test_sniff_full(filename_orig: &path::Path, expected_mime: Mime, supplied_type: Option<Mime>) {
    test_sniff_with_flags(
        filename_orig,
        expected_mime,
        supplied_type,
        NoSniffFlag::Off,
        ApacheBugFlag::Off,
    )
}

#[cfg(test)]
fn test_sniff_classification(file: &str, expected_mime: Mime, supplied_type: Option<Mime>) {
    let mut x = PathBuf::from("./");
    x.push(expected_mime.type_().as_str());
    x.push(expected_mime.subtype().as_str());
    x.push(file);
    test_sniff_full(&x, expected_mime, supplied_type);
}
#[cfg(test)]
fn test_sniff_classification_sup(file: &str, expected_mime: Mime) {
    test_sniff_classification(file, expected_mime.clone(), None);
    let no_sub = format!("{}/", expected_mime.type_()).parse().unwrap();
    test_sniff_classification(file, expected_mime, Some(no_sub));
}

#[test]
fn test_sniff_x_icon() {
    test_sniff_classification_sup("test.ico", "image/x-icon".parse().unwrap());
}

#[test]
fn test_sniff_x_icon_cursor() {
    test_sniff_classification_sup("test_cursor.ico", "image/x-icon".parse().unwrap());
}

#[test]
fn test_sniff_bmp() {
    test_sniff_classification_sup("test.bmp", mime::IMAGE_BMP);
}

#[test]
fn test_sniff_gif87a() {
    test_sniff_classification_sup("test87a", mime::IMAGE_GIF);
}

#[test]
fn test_sniff_gif89a() {
    test_sniff_classification_sup("test89a.gif", mime::IMAGE_GIF);
}

#[test]
fn test_sniff_webp() {
    test_sniff_classification_sup("test.webp", "image/webp".parse().unwrap());
}

#[test]
fn test_sniff_png() {
    test_sniff_classification_sup("test.png", mime::IMAGE_PNG);
}

#[test]
fn test_sniff_jpg() {
    test_sniff_classification_sup("test.jpg", mime::IMAGE_JPEG);
}

#[test]
fn test_sniff_webm() {
    test_sniff_classification_sup("test.webm", "video/webm".parse().unwrap());
}

#[test]
fn test_sniff_mp4() {
    test_sniff_classification_sup("test.mp4", "video/mp4".parse().unwrap());
}

#[test]
fn test_sniff_avi() {
    test_sniff_classification_sup("test.avi", "video/avi".parse().unwrap());
}

#[test]
fn test_sniff_basic() {
    test_sniff_classification_sup("test.au", "audio/basic".parse().unwrap());
}

#[test]
fn test_sniff_aiff() {
    test_sniff_classification_sup("test.aif", "audio/aiff".parse().unwrap());
}

#[test]
fn test_sniff_mpeg() {
    test_sniff_classification_sup("test.mp3", "audio/mpeg".parse().unwrap());
}

#[test]
fn test_sniff_midi() {
    test_sniff_classification_sup("test.mid", "audio/midi".parse().unwrap());
}

#[test]
fn test_sniff_wave() {
    test_sniff_classification_sup("test.wav", "audio/wave".parse().unwrap());
}

#[test]
fn test_sniff_ogg() {
    test_sniff_classification("small.ogg", "application/ogg".parse().unwrap(), None);
    test_sniff_classification(
        "small.ogg",
        "application/ogg".parse().unwrap(),
        Some("audio/".parse().unwrap()),
    );
}

#[test]
#[should_panic]
fn test_sniff_vsn_ms_fontobject() {
    test_sniff_classification_sup(
        "vnd.ms-fontobject",
        "application/vnd.ms-fontobject".parse().unwrap(),
    );
}

#[test]
#[should_panic]
fn test_sniff_true_type() {
    test_sniff_full(
        &PathBuf::from("unknown/true_type.ttf"),
        "(TrueType)/".parse().unwrap(),
        None,
    );
}

#[test]
#[should_panic]
fn test_sniff_open_type() {
    test_sniff_full(
        &PathBuf::from("unknown/open_type"),
        "(OpenType)/".parse().unwrap(),
        None,
    );
}

#[test]
#[should_panic]
fn test_sniff_true_type_collection() {
    test_sniff_full(
        &PathBuf::from("unknown/true_type_collection.ttc"),
        "(TrueType Collection)/".parse().unwrap(),
        None,
    );
}

#[test]
#[should_panic]
fn test_sniff_woff() {
    test_sniff_classification_sup("test.wof", "application/font-woff".parse().unwrap());
}

#[test]
fn test_sniff_gzip() {
    test_sniff_classification("test.gz", "application/x-gzip".parse().unwrap(), None);
}

#[test]
fn test_sniff_zip() {
    test_sniff_classification("test.zip", "application/zip".parse().unwrap(), None);
}

#[test]
fn test_sniff_rar() {
    test_sniff_classification(
        "test.rar",
        "application/x-rar-compressed".parse().unwrap(),
        None,
    );
}

#[test]
fn test_sniff_text_html_doctype_20() {
    test_sniff_classification("text_html_doctype_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_doctype_20_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_doctype_3e() {
    test_sniff_classification("text_html_doctype_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_doctype_3e_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_page_20() {
    test_sniff_classification("text_html_page_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_page_20_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_page_3e() {
    test_sniff_classification("text_html_page_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_page_3e_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_head_20() {
    test_sniff_classification("text_html_head_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_head_20_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_head_3e() {
    test_sniff_classification("text_html_head_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_head_3e_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_script_20() {
    test_sniff_classification("text_html_script_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_script_20_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_script_3e() {
    test_sniff_classification("text_html_script_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_script_3e_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_iframe_20() {
    test_sniff_classification("text_html_iframe_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_iframe_20_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_iframe_3e() {
    test_sniff_classification("text_html_iframe_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_iframe_3e_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_h1_20() {
    test_sniff_classification("text_html_h1_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_h1_20_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_h1_3e() {
    test_sniff_classification("text_html_h1_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_h1_3e_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_div_20() {
    test_sniff_classification("text_html_div_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_div_20_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_div_3e() {
    test_sniff_classification("text_html_div_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_div_3e_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_font_20() {
    test_sniff_classification("text_html_font_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_font_20_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_font_3e() {
    test_sniff_classification("text_html_font_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_font_3e_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_table_20() {
    test_sniff_classification("text_html_table_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_table_20_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_table_3e() {
    test_sniff_classification("text_html_table_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_table_3e_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_a_20() {
    test_sniff_classification("text_html_a_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_a_20_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_a_3e() {
    test_sniff_classification("text_html_a_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_a_3e_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_style_20() {
    test_sniff_classification("text_html_style_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_style_20_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_style_3e() {
    test_sniff_classification("text_html_style_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_style_3e_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_title_20() {
    test_sniff_classification("text_html_title_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_title_20_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_title_3e() {
    test_sniff_classification("text_html_title_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_title_3e_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_b_20() {
    test_sniff_classification("text_html_b_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_b_20_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_b_3e() {
    test_sniff_classification("text_html_b_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_b_3e_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_body_20() {
    test_sniff_classification("text_html_body_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_body_20_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_body_3e() {
    test_sniff_classification("text_html_body_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_body_3e_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_br_20() {
    test_sniff_classification("text_html_br_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_br_20_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_br_3e() {
    test_sniff_classification("text_html_br_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_br_3e_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_p_20() {
    test_sniff_classification("text_html_p_20.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_p_20_u.html", mime::TEXT_HTML, None);
}
#[test]
fn test_sniff_text_html_p_3e() {
    test_sniff_classification("text_html_p_3e.html", mime::TEXT_HTML, None);
    test_sniff_classification("text_html_p_3e_u.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_comment_20() {
    test_sniff_classification("text_html_comment_20.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_text_html_comment_3e() {
    test_sniff_classification("text_html_comment_3e.html", mime::TEXT_HTML, None);
}

#[test]
fn test_sniff_xml() {
    test_sniff_classification("test.xml", mime::TEXT_XML, None);
}

#[test]
fn test_sniff_pdf() {
    test_sniff_classification("test.pdf", mime::APPLICATION_PDF, None);
}

#[test]
fn test_sniff_postscript() {
    test_sniff_classification("test.ps", "application/postscript".parse().unwrap(), None);
}

#[test]
fn test_sniff_utf_16be_bom() {
    test_sniff_classification("utf16bebom.txt", mime::TEXT_PLAIN, None);
}

#[test]
fn test_sniff_utf_16le_bom() {
    test_sniff_classification("utf16lebom.txt", mime::TEXT_PLAIN, None);
}

#[test]
fn test_sniff_utf_8_bom() {
    test_sniff_classification("utf8bom.txt", mime::TEXT_PLAIN, None);
}

#[test]
fn test_sniff_rss_feed() {
    // RSS feeds
    test_sniff_full(
        &PathBuf::from("text/xml/feed.rss"),
        "application/rss+xml".parse().unwrap(),
        Some(mime::TEXT_HTML),
    );
    test_sniff_full(
        &PathBuf::from("text/xml/rdf_rss.xml"),
        "application/rss+xml".parse().unwrap(),
        Some(mime::TEXT_HTML),
    );
    // Not RSS feeds
    test_sniff_full(
        &PathBuf::from("text/xml/rdf_rss_ko_1.xml"),
        mime::TEXT_HTML,
        Some(mime::TEXT_HTML),
    );
    test_sniff_full(
        &PathBuf::from("text/xml/rdf_rss_ko_2.xml"),
        mime::TEXT_HTML,
        Some(mime::TEXT_HTML),
    );
    test_sniff_full(
        &PathBuf::from("text/xml/rdf_rss_ko_3.xml"),
        mime::TEXT_HTML,
        Some(mime::TEXT_HTML),
    );
    test_sniff_full(
        &PathBuf::from("text/xml/rdf_rss_ko_4.xml"),
        mime::TEXT_HTML,
        Some(mime::TEXT_HTML),
    );
}

#[test]
fn test_sniff_atom_feed() {
    test_sniff_full(
        &PathBuf::from("text/xml/feed.atom"),
        "application/atom+xml".parse().unwrap(),
        Some(mime::TEXT_HTML),
    );
}

#[test]
fn test_sniff_binary_file() {
    test_sniff_full(
        &PathBuf::from("unknown/binary_file"),
        mime::APPLICATION_OCTET_STREAM,
        None,
    );
}

#[test]
fn test_sniff_atom_feed_with_no_sniff_flag_on() {
    test_sniff_with_flags(
        &PathBuf::from("text/xml/feed.atom"),
        mime::TEXT_HTML,
        Some(mime::TEXT_HTML),
        NoSniffFlag::On,
        ApacheBugFlag::Off,
    );
}

#[test]
fn test_sniff_with_no_sniff_flag_on_and_apache_flag_on() {
    test_sniff_with_flags(
        &PathBuf::from("text/xml/feed.atom"),
        mime::TEXT_HTML,
        Some(mime::TEXT_HTML),
        NoSniffFlag::On,
        ApacheBugFlag::On,
    );
}

#[test]
fn test_sniff_utf_8_bom_with_apache_flag_on() {
    test_sniff_with_flags(
        &PathBuf::from("text/plain/utf8bom.txt"),
        mime::TEXT_PLAIN,
        Some("dummy/text".parse().unwrap()),
        NoSniffFlag::Off,
        ApacheBugFlag::On,
    );
}

#[test]
fn test_sniff_utf_16be_bom_with_apache_flag_on() {
    test_sniff_with_flags(
        &PathBuf::from("text/plain/utf16bebom.txt"),
        mime::TEXT_PLAIN,
        Some("dummy/text".parse().unwrap()),
        NoSniffFlag::Off,
        ApacheBugFlag::On,
    );
}

#[test]
fn test_sniff_utf_16le_bom_with_apache_flag_on() {
    test_sniff_with_flags(
        &PathBuf::from("text/plain/utf16lebom.txt"),
        mime::TEXT_PLAIN,
        Some("dummy/text".parse().unwrap()),
        NoSniffFlag::Off,
        ApacheBugFlag::On,
    );
}

#[test]
fn test_sniff_octet_stream_apache_flag_on() {
    test_sniff_with_flags(
        &PathBuf::from("unknown/binary_file"),
        mime::APPLICATION_OCTET_STREAM,
        Some("dummy/binary".parse().unwrap()),
        NoSniffFlag::Off,
        ApacheBugFlag::On,
    );
}

#[test]
fn test_sniff_mp4_video_apache_flag_on() {
    test_sniff_with_flags(
        &PathBuf::from("video/mp4/test.mp4"),
        mime::APPLICATION_OCTET_STREAM,
        Some("video/mp4".parse().unwrap()),
        NoSniffFlag::Off,
        ApacheBugFlag::On,
    );
}

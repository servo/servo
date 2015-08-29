/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net::mime_classifier::as_string_option;
use net::mime_classifier::{Mp4Matcher, MIMEClassifier};
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::path::{self, PathBuf};

fn read_file(path: &path::Path) -> io::Result<Vec<u8>> {
    let mut file = try!(File::open(path));
    let mut buffer = Vec::new();

    try!(file.read_to_end(&mut buffer));

    Ok(buffer)
}

#[test]
fn test_sniff_mp4_matcher() {
    let matcher = Mp4Matcher;

    let p = PathBuf::from("parsable_mime/video/mp4/test.mp4");
    let read_result = read_file(&p);

    match read_result {
        Ok(data) => {
            println!("Data Length {:?}", data.len());
            if !matcher.matches(&data) {
                panic!("Didn't read mime type")
            }
        },
        Err(e) => panic!("Couldn't read from file with error {}", e)
    }
}

#[cfg(test)]
fn test_sniff_full(filename_orig: &path::Path, type_string: &str, subtype_string: &str,
                   supplied_type: Option<(&'static str, &'static str)>) {
    let current_working_directory = env::current_dir().unwrap();
    println!("The current directory is {}", current_working_directory.display());

    let mut filename = PathBuf::from("parsable_mime/");
    filename.push(filename_orig);

    let classifier = MIMEClassifier::new();

    let read_result = read_file(&filename);

    match read_result {
        Ok(data) => {
            match classifier.classify(false, false, &as_string_option(supplied_type), &data) {
                Some((parsed_type, parsed_subtp)) => {
                    if (&parsed_type[..] != type_string) ||
                        (&parsed_subtp[..] != subtype_string) {
                            panic!("File {:?} parsed incorrectly should be {}/{}, parsed as {}/{}",
                                   filename, type_string, subtype_string,
                                   parsed_type, parsed_subtp);
                        }
                }
                None => panic!("No classification found for {:?} with supplied type {:?}",
                               filename, supplied_type),
            }
        }
        Err(e) => panic!("Couldn't read from file {:?} with error {}",
                         filename, e),
    }
}

#[cfg(test)]
fn test_sniff_classification(file: &str, type_string: &str, subtype_string: &str,
                             supplied_type: Option<(&'static str, &'static str)>) {
    let mut x = PathBuf::from("./");
    x.push(type_string);
    x.push(subtype_string);
    x.push(file);
    test_sniff_full(&x, type_string, subtype_string, supplied_type);
}
#[cfg(test)]
fn test_sniff_classification_sup(file: &str, type_string: &'static str, subtype_string: &str) {
    test_sniff_classification(file, type_string, subtype_string, None);
    let class_type = Some((type_string, ""));
    test_sniff_classification(file, type_string, subtype_string, class_type);
}

#[test]
fn test_sniff_x_icon() {
    test_sniff_classification_sup("test.ico", "image", "x-icon");
}

#[test]
fn test_sniff_x_icon_cursor() {
    test_sniff_classification_sup("test_cursor.ico", "image", "x-icon");
}

#[test]
fn test_sniff_bmp() {
    test_sniff_classification_sup("test.bmp", "image", "bmp");
}

#[test]
fn test_sniff_gif87a() {
    test_sniff_classification_sup("test87a", "image", "gif");
}

#[test]
fn test_sniff_gif89a() {
    test_sniff_classification_sup("test89a.gif", "image", "gif");
}

#[test]
fn test_sniff_webp() {
    test_sniff_classification_sup("test.webp", "image", "webp");
}

#[test]
fn test_sniff_png() {
    test_sniff_classification_sup("test.png", "image", "png");
}

#[test]
fn test_sniff_jpg() {
    test_sniff_classification_sup("test.jpg", "image", "jpeg");
}

#[test]
fn test_sniff_webm() {
    test_sniff_classification_sup("test.webm", "video", "webm");
}

#[test]
fn test_sniff_mp4() {
    test_sniff_classification_sup("test.mp4", "video", "mp4");
}

#[test]
fn test_sniff_avi() {
    test_sniff_classification_sup("test.avi", "video", "avi");
}

#[test]
fn test_sniff_basic() {
    test_sniff_classification_sup("test.au", "audio", "basic");
}

#[test]
fn test_sniff_aiff() {
    test_sniff_classification_sup("test.aif", "audio", "aiff");
}

#[test]
fn test_sniff_mpeg() {
    test_sniff_classification_sup("test.mp3", "audio", "mpeg");
}

#[test]
fn test_sniff_midi() {
    test_sniff_classification_sup("test.mid", "audio", "midi");
}

#[test]
fn test_sniff_wave() {
    test_sniff_classification_sup("test.wav", "audio", "wave");
}

#[test]
fn test_sniff_ogg() {
    test_sniff_classification("small.ogg", "application", "ogg", None);
    test_sniff_classification("small.ogg", "application", "ogg", Some(("audio", "")));
}

#[test]
#[should_panic]
fn test_sniff_vsn_ms_fontobject() {
    test_sniff_classification_sup("vnd.ms-fontobject", "application", "vnd.ms-fontobject");
}

#[test]
#[should_panic]
fn test_sniff_true_type() {
    test_sniff_full(&PathBuf::from("unknown/true_type.ttf"), "(TrueType)", "", None);
}

#[test]
#[should_panic]
fn test_sniff_open_type() {
    test_sniff_full(&PathBuf::from("unknown/open_type"), "(OpenType)", "", None);
}

#[test]
#[should_panic]
fn test_sniff_true_type_collection() {
    test_sniff_full(&PathBuf::from("unknown/true_type_collection.ttc"), "(TrueType Collection)", "", None);
}

#[test]
#[should_panic]
fn test_sniff_woff() {
    test_sniff_classification_sup("test.wof", "application", "font-woff");
}

#[test]
fn test_sniff_gzip() {
    test_sniff_classification("test.gz", "application", "x-gzip", None);
}

#[test]
fn test_sniff_zip() {
    test_sniff_classification("test.zip", "application", "zip", None);
}

#[test]
fn test_sniff_rar() {
    test_sniff_classification("test.rar", "application", "x-rar-compressed", None);
}

#[test]
fn test_sniff_text_html_doctype_20() {
    test_sniff_classification("text_html_doctype_20.html", "text", "html", None);
    test_sniff_classification("text_html_doctype_20_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_doctype_3e() {
    test_sniff_classification("text_html_doctype_3e.html", "text", "html", None);
    test_sniff_classification("text_html_doctype_3e_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_page_20() {
    test_sniff_classification("text_html_page_20.html", "text", "html", None);
    test_sniff_classification("text_html_page_20_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_page_3e() {
    test_sniff_classification("text_html_page_3e.html", "text", "html", None);
    test_sniff_classification("text_html_page_3e_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_head_20() {
    test_sniff_classification("text_html_head_20.html", "text", "html", None);
    test_sniff_classification("text_html_head_20_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_head_3e() {
    test_sniff_classification("text_html_head_3e.html", "text", "html", None);
    test_sniff_classification("text_html_head_3e_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_script_20() {
    test_sniff_classification("text_html_script_20.html", "text", "html", None);
    test_sniff_classification("text_html_script_20_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_script_3e() {
    test_sniff_classification("text_html_script_3e.html", "text", "html", None);
    test_sniff_classification("text_html_script_3e_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_iframe_20() {
    test_sniff_classification("text_html_iframe_20.html", "text", "html", None);
    test_sniff_classification("text_html_iframe_20_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_iframe_3e() {
    test_sniff_classification("text_html_iframe_3e.html", "text", "html", None);
    test_sniff_classification("text_html_iframe_3e_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_h1_20() {
    test_sniff_classification("text_html_h1_20.html", "text", "html", None);
    test_sniff_classification("text_html_h1_20_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_h1_3e() {
    test_sniff_classification("text_html_h1_3e.html", "text", "html", None);
    test_sniff_classification("text_html_h1_3e_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_div_20() {
    test_sniff_classification("text_html_div_20.html", "text", "html", None);
    test_sniff_classification("text_html_div_20_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_div_3e() {
    test_sniff_classification("text_html_div_3e.html", "text", "html", None);
    test_sniff_classification("text_html_div_3e_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_font_20() {
    test_sniff_classification("text_html_font_20.html", "text", "html", None);
    test_sniff_classification("text_html_font_20_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_font_3e() {
    test_sniff_classification("text_html_font_3e.html", "text", "html", None);
    test_sniff_classification("text_html_font_3e_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_table_20() {
    test_sniff_classification("text_html_table_20.html", "text", "html", None);
    test_sniff_classification("text_html_table_20_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_table_3e() {
    test_sniff_classification("text_html_table_3e.html", "text", "html", None);
    test_sniff_classification("text_html_table_3e_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_a_20() {
    test_sniff_classification("text_html_a_20.html", "text", "html", None);
    test_sniff_classification("text_html_a_20_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_a_3e() {
    test_sniff_classification("text_html_a_3e.html", "text", "html", None);
    test_sniff_classification("text_html_a_3e_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_style_20() {
    test_sniff_classification("text_html_style_20.html", "text", "html", None);
    test_sniff_classification("text_html_style_20_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_style_3e() {
    test_sniff_classification("text_html_style_3e.html", "text", "html", None);
    test_sniff_classification("text_html_style_3e_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_title_20() {
    test_sniff_classification("text_html_title_20.html", "text", "html", None);
    test_sniff_classification("text_html_title_20_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_title_3e() {
    test_sniff_classification("text_html_title_3e.html", "text", "html", None);
    test_sniff_classification("text_html_title_3e_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_b_20() {
    test_sniff_classification("text_html_b_20.html", "text", "html", None);
    test_sniff_classification("text_html_b_20_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_b_3e() {
    test_sniff_classification("text_html_b_3e.html", "text", "html", None);
    test_sniff_classification("text_html_b_3e_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_body_20() {
    test_sniff_classification("text_html_body_20.html", "text", "html", None);
    test_sniff_classification("text_html_body_20_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_body_3e() {
    test_sniff_classification("text_html_body_3e.html", "text", "html", None);
    test_sniff_classification("text_html_body_3e_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_br_20() {
    test_sniff_classification("text_html_br_20.html", "text", "html", None);
    test_sniff_classification("text_html_br_20_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_br_3e() {
    test_sniff_classification("text_html_br_3e.html", "text", "html", None);
    test_sniff_classification("text_html_br_3e_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_p_20() {
    test_sniff_classification("text_html_p_20.html", "text", "html", None);
    test_sniff_classification("text_html_p_20_u.html", "text", "html", None);
}
#[test]
fn test_sniff_text_html_p_3e() {
    test_sniff_classification("text_html_p_3e.html", "text", "html", None);
    test_sniff_classification("text_html_p_3e_u.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_comment_20() {
    test_sniff_classification("text_html_comment_20.html", "text", "html", None);
}

#[test]
fn test_sniff_text_html_comment_3e() {
    test_sniff_classification("text_html_comment_3e.html", "text", "html", None);
}

#[test]
fn test_sniff_xml() {
    test_sniff_classification("test.xml", "text", "xml", None);
}

#[test]
fn test_sniff_pdf() {
    test_sniff_classification("test.pdf", "application", "pdf", None);
}

#[test]
fn test_sniff_postscript() {
    test_sniff_classification("test.ps", "application", "postscript", None);
}

#[test]
fn test_sniff_utf_16be_bom() {
    test_sniff_classification("utf16bebom.txt", "text", "plain", None);
}

#[test]
fn test_sniff_utf_16le_bom() {
    test_sniff_classification("utf16lebom.txt", "text", "plain", None);
}

#[test]
fn test_sniff_utf_8_bom() {
    test_sniff_classification("utf8bom.txt", "text", "plain", None);
}

#[test]
fn test_sniff_rss_feed() {
    // RSS feeds
    test_sniff_full(&PathBuf::from("text/xml/feed.rss"), "application", "rss+xml", Some(("text", "html")));
    test_sniff_full(&PathBuf::from("text/xml/rdf_rss.xml"), "application", "rss+xml", Some(("text", "html")));
    // Not RSS feeds
    test_sniff_full(&PathBuf::from("text/xml/rdf_rss_ko_1.xml"), "text", "html", Some(("text", "html")));
    test_sniff_full(&PathBuf::from("text/xml/rdf_rss_ko_2.xml"), "text", "html", Some(("text", "html")));
    test_sniff_full(&PathBuf::from("text/xml/rdf_rss_ko_3.xml"), "text", "html", Some(("text", "html")));
    test_sniff_full(&PathBuf::from("text/xml/rdf_rss_ko_4.xml"), "text", "html", Some(("text", "html")));
}

#[test]
fn test_sniff_atom_feed() {
    test_sniff_full(&PathBuf::from("text/xml/feed.atom"), "application", "atom+xml", Some(("text", "html")));
}

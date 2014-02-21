/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use extra::url;
use extra::url::Url;
use std::hashmap::HashMap;
use std::os;

/**
Create a URL object from a string. Does various helpful browsery things like

* If there's no current url and the path looks like a file then it will
  create a file url based of the current working directory
* If there's a current url and the new path is relative then the new url
  is based off the current url

*/
// TODO: about:failure->
pub fn parse_url(str_url: &str, base_url: Option<Url>) -> Url {
    let str_url = str_url.trim_chars(& &[' ', '\t', '\n', '\r', '\x0C']).to_owned();
    let schm = url::get_scheme(str_url);
    let str_url = match schm {
        Err(_) => {
            if base_url.is_none() {
                // Assume we've been given a file path. If it's absolute just return
                // it, otherwise make it absolute with the cwd.
                if str_url.starts_with("/") {
                    ~"file://" + str_url
                } else {
                    let mut path = os::getcwd();
                    path.push(str_url);
                    // FIXME (#1094): not the right way to transform a path
                    ~"file://" + path.display().to_str()
                }
            } else {
                let base_url = base_url.unwrap();
                debug!("parse_url: base_url: {:?}", base_url);

                let mut new_url = base_url.clone();
                new_url.query = ~[];
                new_url.fragment = None;

                if str_url.starts_with("//") {
                    new_url.scheme + ":" + str_url
                } else if base_url.path.is_empty() || str_url.starts_with("/") {
                    new_url.path = ~"/";
                    new_url.to_str() + str_url.trim_left_chars(&'/')
                } else if str_url.starts_with("#") {
                    new_url.to_str() + str_url
                } else { // relative path
                    let base_path = base_url.path.trim_right_chars(&|c: char| c != '/');
                    new_url.path = base_path.to_owned();
                    new_url.to_str() + str_url
                }
            }
        },
        Ok((scheme, page)) => {
            match scheme.as_slice() {
                "about" => {
                    match page.as_slice() {
                        "crash" => {
                            fail!("about:crash");
                        }
                        "failure" => {
                            let mut path = os::self_exe_path().expect("can't get exe path");
                            path.push("../src/test/html/failure.html");
                            // FIXME (#1094): not the right way to transform a path
                            ~"file://" + path.display().to_str()
                        }
                        // TODO: handle the rest of the about: pages
                        _ => str_url
                    }
                },
                "data" => {
                    // Drop whitespace within data: URLs, e.g. newlines within a base64
                    // src="..." block.  Whitespace intended as content should be
                    // %-encoded or base64'd.
                    str_url.chars().filter(|&c| !c.is_whitespace()).collect()
                },
                _ => str_url
            }
        }
    };

    // FIXME: Need to handle errors
    url::from_str(str_url).ok().expect("URL parsing failed")
}

#[cfg(test)]
mod parse_url_tests {
    use super::parse_url;
    use std::os;

    #[test]
    fn should_create_absolute_file_url_if_base_url_is_none_and_str_url_looks_filey() {
        let file = "local.html";
        let url = parse_url(file, None);
        debug!("url: {:?}", url);
        assert!(url.scheme == ~"file");
        let path = os::getcwd();
        // FIXME (#1094): not the right way to transform a path
        assert!(url.path.contains(path.display().to_str()));
    }

    #[test]
    fn should_create_url_based_on_old_url_1() {
        let old_str = "http://example.com";
        let old_url = parse_url(old_str, None);
        let new_str = "index.html";
        let new_url = parse_url(new_str, Some(old_url));
        assert!(new_url.scheme == ~"http");
        assert!(new_url.host == ~"example.com");
        assert!(new_url.path == ~"/index.html");
    }

    #[test]
    fn should_create_url_based_on_old_url_2() {
        let old_str = "http://example.com/";
        let old_url = parse_url(old_str, None);
        let new_str = "index.html";
        let new_url = parse_url(new_str, Some(old_url));
        assert!(new_url.scheme == ~"http");
        assert!(new_url.host == ~"example.com");
        assert!(new_url.path == ~"/index.html");
    }

    #[test]
    fn should_create_url_based_on_old_url_3() {
        let old_str = "http://example.com/index.html";
        let old_url = parse_url(old_str, None);
        let new_str = "crumpet.html";
        let new_url = parse_url(new_str, Some(old_url));
        assert!(new_url.scheme == ~"http");
        assert!(new_url.host == ~"example.com");
        assert!(new_url.path == ~"/crumpet.html");
    }

    #[test]
    fn should_create_url_based_on_old_url_4() {
        let old_str = "http://example.com/snarf/index.html";
        let old_url = parse_url(old_str, None);
        let new_str = "crumpet.html";
        let new_url = parse_url(new_str, Some(old_url));
        assert!(new_url.scheme == ~"http");
        assert!(new_url.host == ~"example.com");
        assert!(new_url.path == ~"/snarf/crumpet.html");
    }

    #[test]
    fn should_create_url_based_on_old_url_5() {
        let old_str = "http://example.com/index.html";
        let old_url = parse_url(old_str, None);
        let new_str = "#top";
        let new_url = parse_url(new_str, Some(old_url));

        assert!(new_url.scheme == ~"http");
        assert!(new_url.host == ~"example.com");
        assert!(new_url.path == ~"/index.html");
        assert!(new_url.fragment == Some(~"top"));
    }

    #[test]
    fn should_create_url_based_on_old_url_6() {
        use extra::url::UserInfo;

        let old_str = "http://foo:bar@example.com:8080/index.html";
        let old_url = parse_url(old_str, None);
        let new_str = "#top";
        let new_url = parse_url(new_str, Some(old_url));

        assert!(new_url.scheme == ~"http");
        assert!(new_url.user == Some(UserInfo { user: ~"foo", pass: Some(~"bar") }));
        assert!(new_url.host == ~"example.com");
        assert!(new_url.port == Some(~"8080"));
        assert!(new_url.path == ~"/index.html");
        assert!(new_url.fragment == Some(~"top"));
    }

    #[test]
    fn should_create_url_based_on_old_url_7() {
        let old_str = "https://example.com/snarf/index.html";
        let old_url = parse_url(old_str, None);
        let new_str = "//example.com/crumpet.html";
        let new_url = parse_url(new_str, Some(old_url));
        assert!(new_url.scheme == ~"https");
        assert!(new_url.host == ~"example.com");
        assert!(new_url.path == ~"/crumpet.html");
    }

}

pub type UrlMap<T> = HashMap<Url, T>;

pub fn url_map<T: Clone + 'static>() -> UrlMap<T> {
    HashMap::new()
}


pub fn is_image_data(uri: &str) -> bool {
    static types: &'static [&'static str] = &[&"data:image/png", &"data:image/gif", &"data:image/jpeg"];
    types.iter().any(|&type_| uri.starts_with(type_))
}



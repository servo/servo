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
pub fn make_url(str_url: ~str, current_url: Option<Url>) -> Url {
    let schm = url::get_scheme(str_url);
    let str_url = match schm {
        Err(_) => {
            if current_url.is_none() {
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
                let current_url = current_url.unwrap();
                debug!("make_url: current_url: {:?}", current_url);
                if str_url.starts_with("//") {
                    current_url.scheme + ":" + str_url
                } else if current_url.path.is_empty() ||
                    str_url.starts_with("/") {
                    current_url.scheme + "://" +
                        current_url.host + "/" +
                        str_url.trim_left_chars(&'/')
                } else {
                    let mut path = ~[];
                    for p in current_url.path.split_iter('/') {
                        path.push(p.to_str());
                    }
                    let path = path.init();
                    let mut path = path.iter().map(|x| (*x).clone()).collect::<~[~str]>();
                    path.push(str_url);
                    let path = path.connect("/");
                    
                    current_url.scheme + "://" + current_url.host + path
                }
            }
        },
        Ok((scheme, page)) => {
            match scheme {
                ~"about" => {
                    match page {
                        ~"failure" => {
                            let mut path = os::getcwd();
                            path.push("../src/test/html/failure.html");
                            // FIXME (#1094): not the right way to transform a path
                            ~"file://" + path.display().to_str()
                        }
                        // TODO: handle the rest of the about: pages
                        _ => str_url
                    }
                },
                ~"data" => {
                    // Drop whitespace within data: URLs, e.g. newlines within a base64
                    // src="..." block.  Whitespace intended as content should be
                    // %-encoded or base64'd.
                    str_url.iter().filter(|&c| !c.is_whitespace()).collect()
                },
                _ => str_url
            }
        }
    };

    // FIXME: Need to handle errors
    url::from_str(str_url).unwrap()
}

#[cfg(test)]
mod make_url_tests {
    use super::make_url;
    use std::os;

    #[test]
    fn should_create_absolute_file_url_if_current_url_is_none_and_str_url_looks_filey() {
        let file = ~"local.html";
        let url = make_url(file, None);
        debug!("url: {:?}", url);
        assert!(url.scheme == ~"file");
        let path = os::getcwd();
        // FIXME (#1094): not the right way to transform a path
        assert!(url.path.contains(path.display().to_str()));
    }

    #[test]
    fn should_create_url_based_on_old_url_1() {
        let old_str = ~"http://example.com";
        let old_url = make_url(old_str, None);
        let new_str = ~"index.html";
        let new_url = make_url(new_str, Some(old_url));
        assert!(new_url.scheme == ~"http");
        assert!(new_url.host == ~"example.com");
        assert!(new_url.path == ~"/index.html");
    }

    #[test]
    fn should_create_url_based_on_old_url_2() {
        let old_str = ~"http://example.com/";
        let old_url = make_url(old_str, None);
        let new_str = ~"index.html";
        let new_url = make_url(new_str, Some(old_url));
        assert!(new_url.scheme == ~"http");
        assert!(new_url.host == ~"example.com");
        assert!(new_url.path == ~"/index.html");
    }

    #[test]
    fn should_create_url_based_on_old_url_3() {
        let old_str = ~"http://example.com/index.html";
        let old_url = make_url(old_str, None);
        let new_str = ~"crumpet.html";
        let new_url = make_url(new_str, Some(old_url));
        assert!(new_url.scheme == ~"http");
        assert!(new_url.host == ~"example.com");
        assert!(new_url.path == ~"/crumpet.html");
    }

    #[test]
    fn should_create_url_based_on_old_url_4() {
        let old_str = ~"http://example.com/snarf/index.html";
        let old_url = make_url(old_str, None);
        let new_str = ~"crumpet.html";
        let new_url = make_url(new_str, Some(old_url));
        assert!(new_url.scheme == ~"http");
        assert!(new_url.host == ~"example.com");
        assert!(new_url.path == ~"/snarf/crumpet.html");
    }

}

pub type UrlMap<T> = HashMap<Url, T>;

pub fn url_map<T: Clone + 'static>() -> UrlMap<T> {
    HashMap::new()
}


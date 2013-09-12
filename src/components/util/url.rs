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
                    ~"file://" + os::getcwd().push(str_url).to_str()
                }
            } else {
                let current_url = current_url.unwrap();
                debug!("make_url: current_url: %?", current_url);
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
                        ~"failure" => ~"file://" + os::getcwd().push("../src/test/html/failure.html").to_str(),
                        // TODO: handle the rest of the about: pages
                        _ => str_url
                    }
                },
                _ => str_url
            }
        }
    };

    // FIXME: Need to handle errors
    url::from_str(str_url).unwrap()
}

mod make_url_tests {

    #[test]
    fn should_create_absolute_file_url_if_current_url_is_none_and_str_url_looks_filey() {
        let file = ~"local.html";
        let url = make_url(file, None);
        debug!("url: %?", url);
        assert!(url.scheme == ~"file");
        assert!(url.path.contains(os::getcwd().to_str()));
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

pub type UrlMap<T> = @mut HashMap<Url, T>;

pub fn url_map<T: Clone + 'static>() -> UrlMap<T> {
    @mut HashMap::new()
}

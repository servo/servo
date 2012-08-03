export make_url;

import std::net::url;
import url::{get_scheme, url};

/**
Create a URL object from a string. Does various helpful browsery things like

* If there's no current url and the path looks like a file then it will
  create a file url based of the current working directory
* If there's a current url and the new path is relative then the new url
  is based off the current url

*/
#[allow(non_implicitly_copyable_typarams)]
fn make_url(str_url: ~str, current_url: option<url>) -> url {
    let mut schm = get_scheme(str_url);
    let str_url = if result::is_err(schm) {
        if current_url.is_none() {
            // If all we have is a filename, assume it's a local relative file
            // and build an absolute path with the cwd
            ~"file://" + path::connect(os::getcwd(), str_url)
        } else {
            let current_url = current_url.get();
            #debug("make_url: current_url: %?", current_url);
            if current_url.path.is_empty() || current_url.path.ends_with("/") {
                current_url.scheme + "://" + path::connect(current_url.host, str_url)
            } else {
                let path = path::split(current_url.path);
                let path = path.init();
                let path = path::connect_many(path + ~[copy str_url]);

                current_url.scheme + "://" + path::connect(current_url.host, path)
            }
        }
    } else {
        copy str_url
    };

    // FIXME: Need to handle errors
    url::from_str(str_url).get()
}

#[test]
fn should_create_absolute_file_url_if_current_url_is_none_and_str_url_looks_filey() {
    let file = ~"local.html";
    let url = make_url(file, none);
    #debug("url: %?", url);
    assert url.scheme == ~"file";
    assert url.path.contains(os::getcwd());
}

#[test]
fn should_create_url_based_on_old_url_1() {
    let old_str = ~"http://example.com";
    let old_url = make_url(old_str, none);
    let new_str = ~"index.html";
    let new_url = make_url(new_str, some(old_url));
    assert new_url.scheme == ~"http";
    assert new_url.host == ~"example.com";
    assert new_url.path == ~"/index.html";
}

#[test]
fn should_create_url_based_on_old_url_2() {
    let old_str = ~"http://example.com/";
    let old_url = make_url(old_str, none);
    let new_str = ~"index.html";
    let new_url = make_url(new_str, some(old_url));
    assert new_url.scheme == ~"http";
    assert new_url.host == ~"example.com";
    assert new_url.path == ~"/index.html";
}

#[test]
fn should_create_url_based_on_old_url_3() {
    let old_str = ~"http://example.com/index.html";
    let old_url = make_url(old_str, none);
    let new_str = ~"crumpet.html";
    let new_url = make_url(new_str, some(old_url));
    assert new_url.scheme == ~"http";
    assert new_url.host == ~"example.com";
    assert new_url.path == ~"/crumpet.html";
}

#[test]
fn should_create_url_based_on_old_url_4() {
    let old_str = ~"http://example.com/snarf/index.html";
    let old_url = make_url(old_str, none);
    let new_str = ~"crumpet.html";
    let new_url = make_url(new_str, some(old_url));
    assert new_url.scheme == ~"http";
    assert new_url.host == ~"example.com";
    assert new_url.path == ~"/snarf/crumpet.html";
}

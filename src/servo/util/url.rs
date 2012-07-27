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
fn make_url(str_url: ~str, current_url: option<url>) -> url {
    let str_url = if get_scheme(str_url).is_some() {
        str_url
    } else {
        if current_url.is_none() {
            // If all we have is a filename, assume it's a local relative file
            // and build an absolute path with the cwd
            ~"file://" + path::connect(os::getcwd(), str_url)
        } else {
            fail;//current_url.get().scheme + "://" + str_url
        }
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

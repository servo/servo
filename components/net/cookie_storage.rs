use url::Url;
use cookie::Cookie;

pub struct CookieStorage {
    cookies: Vec<Cookie>
}

impl CookieStorage {
    pub fn new() -> CookieStorage {
        CookieStorage {
            cookies: Vec::new()
        }
    }

    pub fn push(&mut self, cookie: Cookie) {
        match self.cookies.iter().position(|c| c.domain == cookie.domain && c.path == cookie.path && c.name == cookie.name) {
            Some(ind) => { self.cookies.remove(ind); }
            None => {}
        };

        self.cookies.push(cookie);
    }

    pub fn cookies_for_url(&mut self, url: Url) -> Option<String> {
        let filterer = |&:c: &&mut Cookie| -> bool {
            error!(" === SENT COOKIE : {} {} {} {}", c.name, c.value, c.domain, c.path);
            error!(" === SENT COOKIE RESULT {}", c.appropriate_for_url(url.clone()));
            c.appropriate_for_url(url.clone())
        };
        let mut url_cookies = self.cookies.iter_mut().filter(filterer);
        let reducer = |&:acc: String, c: &mut Cookie| -> String {
            c.touch();
            (match acc.len() {
                0 => acc,
                _ => acc + ";"
            }) + c.name.as_slice() + "=" + c.value.as_slice()
        };
        let result = url_cookies.fold("".to_string(), reducer);

        error!(" === COOKIES SENT: {}", result);
        match result.len() {
            0 => None,
            _ => Some(result)
        }
    }
}

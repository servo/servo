const SERVER_LOCATION = "resources";
const SERVER_SCRIPT = SERVER_LOCATION + "/cookie-setter.py";

function stripPrefixAndWhitespace(cookie_text) {
  return cookie_text.replace(/^Cookie: /, '').replace(/^\s+|\s+$/g, '');
}

function getLocalResourcesPath() {
  return location.pathname.replace(/[^\/]*$/, "") + SERVER_LOCATION;
}

function getAbsoluteServerLocation() {
  return getLocalResourcesPath().replace(/resources.*$/,'')+ SERVER_SCRIPT;
}

function expireCookie(name, expiry_date, path) {
  name = name || "";
  expiry_date = expiry_date || "Thu, 01 Jan 1970 00:00:00 UTC";
  path = path || getLocalResourcesPath();
  document.cookie = name + "=; expires=" + expiry_date + "; path=" + path + ";";
}

function CookieManager() {
  this.initial_cookies = [];
}

CookieManager.prototype.parse = document_cookies => {
  this.initial_cookies = [];
  document_cookies = document_cookies.replace(/^Cookie: /, '');
  if (document_cookies != "") {
    this.initial_cookies = document_cookies.split(/\s*;\s*/);
  }
}

CookieManager.prototype.diffWith = document_cookies => {
  this.actual_cookies = document_cookies;
  for (let i in initial_cookies) {
    let no_spaces_cookie_regex =
        new RegExp(/\s*[\;]*\s/.source + initial_cookies[i]);
    this.actual_cookies = actual_cookies.replace(no_spaces_cookie_regex, '');
  }
  return this.actual_cookies;
}

CookieManager.prototype.resetCookies = () => {
  expireCookie(/*name=*/"");  // If a cookie without keys was accepted, drop it.
  if (this.actual_cookies == "") {
    return;  // There is nothing to reset here.
  }
  let cookies_to_delete = this.actual_cookies.split(/\s*;\s*/)
  for (let i in cookies_to_delete){
    expireCookie(cookies_to_delete[i].replace(/=.*$/, ""));
    // Drop cookies with same name that were set to the root path which happens
    // for example due to "0010" still failing.
    expireCookie(cookies_to_delete[i].replace(/=.*$/, ""),
                 /*expiry_date=*/null,
                 /*path=*/'/');
  }
}

function createCookieTest(file) {
  return t => {
    const iframe = document.createElement('iframe');
    document.body.appendChild(iframe);
    let diff_tool = new CookieManager();
    t.add_cleanup(diff_tool.resetCookies);
    return new Promise((resolve, reject) => {
      diff_tool.parse(document.cookie);
      window.addEventListener("message", t.step_func(e => {
        assert_true(!!e.data, "Message contains data")
        resolve(e.data);
      }));
      iframe.src = getAbsoluteServerLocation() + "?file=" + file;
    }).then((response) => {
      let actual_cookies = diff_tool.diffWith(response.cookies);
      let expected_cookies = stripPrefixAndWhitespace(response.expectation);
      assert_equals(actual_cookies, expected_cookies);
    });
  }
};

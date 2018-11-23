const SERVER_LOCATION = "resources";
const SERVER_SCRIPT = SERVER_LOCATION + "/cookie-setter.py";

/* Adds a div with "<time> [<tag>] - <message>" to the "#log" container.*/
function log(message, tag) {
  let log_str = document.createElement('div');
  log_str.textContent = new Date().toTimeString().replace(/\s.+$/, '');
  if (tag) {
    log_str.textContent += " [" + tag + "] ";
  }
  log_str.textContent += " - " + message;
  let log_container = document.getElementById("log");
  log_container.appendChild(log_str);
  log_container.scrollTo(0, log_container.scrollHeight);
}

/* Removes the "Cookie: " prefix and strip any leading or trailing whitespace.*/
function stripPrefixAndWhitespace(cookie_text) {
  return cookie_text.replace(/^Cookie: /, '').replace(/^\s+|\s+$/g, '');
}

/* Returns the absolute path of the resource folder, ignoring any navigation. */
function getLocalResourcesPath() {
  let replace = "(" + SERVER_LOCATION + "\/)*";  // Redundant location.
  replace += "[^\/]*$";  // Everything after the last "/".
  return location.pathname.replace(new RegExp(replace), "") + SERVER_LOCATION;
}

/* Returns the absolute server location ignoring any navgation.*/
function getAbsoluteServerLocation() {
  // Replace the server location and everything coming after it ...
  let replace = SERVER_LOCATION + ".*$";
  // ... with the Server script (which includes the server location).
  return getLocalResourcesPath().replace(new RegExp(replace),'')+ SERVER_SCRIPT;
}

/* Expires a cookie by name by setting it's expiry date into the past.*/
function expireCookie(name, expiry_date, path) {
  name = name || "";
  expiry_date = expiry_date || "Thu, 01 Jan 1970 00:00:00 UTC";
  path = path || getLocalResourcesPath();
  document.cookie = name + "=; expires=" + expiry_date + "; path=" + path + ";";
}

/* Captures a snapshot of cookies with |parse| and allows to diff it with a
second snapshot with |diffWith|. This allows to run tests even if cookies were
previously set that would mess with the expected final set of Cookies.
With |resetCookies|, all cookies set between first and second snapshot are
expired. */
function CookieManager() {
  this.initial_cookies = [];
}

/* Creates a snapshot of the current given document cookies.*/
CookieManager.prototype.parse = document_cookies => {
  this.initial_cookies = [];
  document_cookies = document_cookies.replace(/^Cookie: /, '');
  if (document_cookies != "") {
    this.initial_cookies = document_cookies.split(/\s*;\s*/);
  }
}

/* Creates a diff of newly added cookies between the initial snapshot and the
newly given cookies. The diff is stored for cleaning purposes. A second call
will replace the the stored diff entirely.*/
CookieManager.prototype.diffWith = document_cookies => {
  this.actual_cookies = document_cookies;
  for (let i in initial_cookies) {
    let no_spaces_cookie_regex =
        new RegExp(/\s*[\;]*\s/.source + initial_cookies[i].replace(/\\/, "\\\\"));
    this.actual_cookies = this.actual_cookies.replace(no_spaces_cookie_regex, '');
  }
  return this.actual_cookies;
}

/* Cleans cookies between the first and the second snapshot.
Some tests might set cookies to the root path or cookies without key. Both cases
are dropped here.*/
CookieManager.prototype.resetCookies = () => {
  // If a cookie without keys was accepted, drop it additionally.
  let cookies_to_delete = [""].concat(this.actual_cookies.split(/\s*;\s*/))
  for (let i in cookies_to_delete){
    expireCookie(cookies_to_delete[i].replace(/=.*$/, ""));
    // Drop cookies with same name that were set to the root path which happens
    // for example due to "0010" still failing.
    expireCookie(cookies_to_delete[i].replace(/=.*$/, ""),
                 /*expiry_date=*/null,
                 /*path=*/'/');
    // Some browsers incorrectly include the final "forward slash" character
    // when calculating the default path. The expected behavior for default
    // path calculation is verified elsewhere; this utility accommodates the
    // non-standard behavior in order to improve the focus of the test suite.
    expireCookie(cookies_to_delete[i].replace(/=.*$/, ""),
                 /*expiry_date=*/null,
                 /*path=*/getLocalResourcesPath() + "/");
  }
}

/* Returns a new cookie test.
The test creates an iframe where a |file| from the cookie-setter.py is loaded.
This sets cookies which are diffed with an initial cookie snapshot and compared
to the expectation that the server returned.
Finally, it cleans up newly set cookies and all cookies in the root path or
without key. */
function createCookieTest(file) {
  return t => {
    let iframe_container = document.getElementById("iframes");
    const iframe = document.createElement('iframe');
    iframe_container.appendChild(iframe);
    iframe_container.scrollTo(0, iframe_container.scrollHeight);
    let diff_tool = new CookieManager();
    t.add_cleanup(diff_tool.resetCookies);
    return new Promise((resolve, reject) => {
      diff_tool.parse(document.cookie);
      if (diff_tool.initial_cookies.length > 0) {
        // The cookies should ideally be empty. If that isn't the case, log it.
        //Cookies with equal keys (e.g. foo=) may have unwanted side-effects.
        log("Run with existing cookies: " + diff_tool.initial_cookies, file);
      }
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

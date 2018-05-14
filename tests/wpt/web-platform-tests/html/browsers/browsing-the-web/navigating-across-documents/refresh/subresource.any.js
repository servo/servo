promise_test(() => {
  return fetch("resources/refresh.py").then(response => {
    assert_equals(response.headers.get("refresh"), "0;./refreshed.txt?\u0080\u00FF"); // Make sure bytes got mapped to code points of the same value
    assert_equals(response.url, (new URL("resources/refresh.py", self.location)).href);
  });
}, "Refresh does not affect subresources.");

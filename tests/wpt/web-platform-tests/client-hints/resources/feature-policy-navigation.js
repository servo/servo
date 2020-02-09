const test_frame = (origin, hints, allow, message, url = "/client-hints/resources/expect-client-hints-headers-iframe.py?") => {
  promise_test(() => {
    return new Promise((resolve, reject) => {
      let frame = document.createElement('iframe');
      frame.src = get_host_info()[origin] + url + hints;
      frame.allow = allow;
      window.addEventListener('message', function(e) {
        try {
          assert_equals(typeof e.data, "string");
          assert_equals(e.data, "PASS");
        } catch {
          reject(e.data);
        }
        resolve();
      });
      document.body.appendChild(frame);
    });
  }, message);
}

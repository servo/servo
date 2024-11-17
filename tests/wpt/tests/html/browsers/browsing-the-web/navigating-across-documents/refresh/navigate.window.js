async_test(t => {
  var loadCount = 0;
  var expectedReferrer = location.href;
  const frame = document.createElement("iframe");
  var originalPath = "resources/refresh.py";
  frame.src = originalPath;
  frame.onload = t.step_func(() => {
    loadCount++;
    if (loadCount === 1) {
      assert_equals(frame.contentWindow.location.href, new URL(originalPath, self.location).href, "original page loads");
      assert_equals(frame.contentDocument.referrer, expectedReferrer, "referrer is parent frame");
      expectedReferrer = frame.src;
    } else if (loadCount === 2) {
      assert_equals(frame.contentWindow.location.href,
        new URL("resources/refreshed.txt?\u0080\u00FF", self.location).href, "bytes got mapped to code points of the same value");
      assert_equals(frame.contentDocument.referrer, expectedReferrer, "referrer is previous page");
      t.done();
    }
  });
  document.body.appendChild(frame)
}, "When navigating the Refresh header needs to be followed");

async_test(t => {
  var loadCount = 0;
  var expectedReferrer = location.href;
  const frame = document.createElement("iframe");
  var originalPath = "resources/multiple.asis";
  frame.src = originalPath
  frame.onload = t.step_func(() => {
    loadCount++;
    if (loadCount === 1) {
      assert_equals(frame.contentWindow.location.href, new URL(originalPath, self.location).href, "original page loads");
      assert_equals(frame.contentDocument.referrer, expectedReferrer, "referrer is parent frame");
      expectedReferrer = frame.src;
    } else if (loadCount === 2) {
      assert_equals(frame.contentWindow.location.href, new URL("resources/refreshed.txt", self.location).href, "refresh page loads");
      assert_equals(frame.contentDocument.referrer, expectedReferrer, "referrer is previous page");
      t.done();
    }
  });
  document.body.appendChild(frame)
}, "When there's both a Refresh header and <meta> the Refresh header wins")

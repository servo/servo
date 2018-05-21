// META: script=/common/media.js

const videoURL = getVideoURI("/images/pattern"),
      videoMIMEType = getMediaContentType(videoURL);

[
  [videoURL, videoMIMEType, "video"],
  ["/images/red.png", "image/png", "image"],
  ["/common/text-plain.txt", "text/plain", "text"],
  ["/common/blank.html", "text/html", "HTML"]
].forEach(val => {
  async_test(t => {
    const frame = document.body.appendChild(document.createElement("iframe"));
    t.add_cleanup(() => frame.remove());
    frame.src = val[0];
    frame.onload = t.step_func_done(() => {
      assert_equals(frame.contentDocument.contentType, val[1]);
      frame.contentDocument.write("<b>Heya</b>");
      assert_equals(frame.contentDocument.body.firstChild.localName, "b");
      assert_equals(frame.contentDocument.body.firstChild.textContent, "Heya");
      assert_equals(frame.contentDocument.contentType, val[1]);

      // Make sure a load event is fired across browsers
      // https://github.com/w3c/web-platform-tests/pull/10239
      frame.contentDocument.close();
    });
  }, "document.write(): " + val[2] + " document");
});

// Verify that navigating an about:blank document to a javascript: URL that
// creates a new document, copies over the base URL from the old document to the
// new document.
onload = () => {
  async_test((t) => {
    const frame = document.createElement('iframe');

    frame.onload = () => {
      assert_equals(document.baseURI, frame.contentDocument.baseURI);

      // We'll need to monitor onload again for the javascript: navigation.
      frame.onload = () => {
        assert_equals(document.baseURI, frame.contentDocument.baseURI);
        assert_equals('foo', frame.contentDocument.body.textContent);
      };
      frame.src = "javascript:'foo'";
      t.done();
    };

    document.body.appendChild(frame);
  }, "javascript: url nav base url test");
};

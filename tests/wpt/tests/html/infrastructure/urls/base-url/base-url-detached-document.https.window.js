// Verify that an about:blank or about:srcdoc document remembers the baseURI
// it was created with even after it's detached.
const runTest = (frame_type) => {
  async_test((t) => {
    const frame = document.createElement('iframe');

    if (frame_type == "about:blank")
      frame.src = "about:blank";
    else
      frame.srcdoc = "foo";

    frame.onload = () => {
      const frame_doc = frame.contentDocument;
      const initial_base_uri = document.baseURI;
      assert_equals(initial_base_uri, frame_doc.baseURI);

      const base_element = document.createElement('base');
      base_element.href = "https://example.com";
      document.head.appendChild(base_element);
      assert_equals(initial_base_uri, frame_doc.baseURI);

      frame.remove();
      assert_equals(initial_base_uri, frame_doc.baseURI);
      t.done();
    };

    document.body.appendChild(frame);
  }, frame_type);
};

onload = () => {
  runTest("about:blank");
  runTest("about:srcdoc");
};

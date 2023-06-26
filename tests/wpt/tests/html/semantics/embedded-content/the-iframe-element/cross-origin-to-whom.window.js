async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  frame.src = "support/document-with-embedded-svg.html";
  const elements = {
    "embed": ["getSVGDocument"],
    "frame": ["contentDocument"],
    "iframe": ["getSVGDocument", "contentDocument"],
    "object": ["getSVGDocument", "contentDocument"]
  };
  function assert_apis(instance) {
    const name = instance.localName;
    let priorPossibleDocument = null;
    elements[name].forEach(api => {
      const possibleDocument = api == "getSVGDocument" ? instance[api]() : instance[api];
      assert_not_equals(possibleDocument, null, `${name}.${api}`);
      // This needs standardizing still
      // assert_class_string(possibleDocument, "XMLDocument");

      // Ensure getSVGDocument() and contentDocument if both available return the same
      if (priorPossibleDocument === null) {
        priorPossibleDocument = possibleDocument;
      } else {
        assert_equals(priorPossibleDocument, possibleDocument);
      }
    });
  }
  frame.onload = t.step_func_done(() => {
    const instances = Object.keys(elements).map(element => frame.contentDocument.querySelector(element));
    // Everything is same origin and same origin-domain, no sweat
    instances.forEach(instance => assert_apis(instance));
    // Make the current settings object cross origin-domain (SVG and its container are not affected)
    document.domain = document.domain;
    assert_equals(frame.contentDocument, null);
    instances.forEach(instance => assert_apis(instance));
  });
  document.body.appendChild(frame);
}, "Test embed/frame/iframe/object nested document APIs for same origin-domain and cross origin-domain current settings object");

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  frame.src = "support/document-with-embedded-svg.html";
  const elements = {
    "embed": ["getSVGDocument"],
    "frame": ["contentDocument"],
    "iframe": ["getSVGDocument", "contentDocument"],
    "object": ["getSVGDocument", "contentDocument"]
  };
  function element_to_document(element, api) {
    return api === "getSVGDocument" ? element[api]() : element[api];
  }
  function assert_apis(instance, assertNull = false) {
    const name = instance.localName;
    let priorPossibleDocument = null;
    elements[name].forEach(api => {
      const assertReason = `${name}.${api}`;
      const possibleDocument = element_to_document(instance, api);
      if (assertNull) {
        assert_equals(possibleDocument, null, assertReason);
        return;
      } else {
        assert_not_equals(possibleDocument, null, assertReason);

        // This needs standardizing still
        // assert_class_string(possibleDocument, "XMLDocument");
      }

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
    // Make the SVG cross origin-domain (its container and the current settings object are not
    // affected)
    instances.forEach(instance => {
      const svgDocument = element_to_document(instance, elements[instance.localName][0]);
      svgDocument.domain = svgDocument.domain;
    });
    instances.forEach(instance => assert_apis(instance, true));
    const svgContainer = frame.contentDocument;
    // Make the current settings object same origin-domain with the SVG and cross origin-domain with
    // SVG's container (SVG's container is not affected)
    document.domain = document.domain;
    assert_equals(frame.contentDocument, null);
    instances.forEach(instance => assert_apis(instance, true));
    // Make everything same origin-domain once more
    svgContainer.domain = svgContainer.domain;
    instances.forEach(instance => assert_apis(instance));
  });
  document.body.appendChild(frame);
}, "Test embed/frame/iframe/object nested document APIs for same origin-domain and cross origin-domain embedder document");

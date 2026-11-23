// Target document has a scoped registry; the element or shadow root has an explicit registry

test(t => {
  const contentDocument = document.implementation.createHTMLDocument();
  const scopedDocRegistry = new CustomElementRegistry();
  scopedDocRegistry.initialize(contentDocument);
  assert_equals(contentDocument.customElementRegistry, scopedDocRegistry);

  const element = document.createElement("div", { customElementRegistry: customElements });
  const clone = contentDocument.importNode(element);
  assert_equals(clone.customElementRegistry, null);
}, "Cloning with explicit global registry (scoped registry target)");

test(t => {
  const contentDocument = document.implementation.createHTMLDocument();
  const scopedDocRegistry = new CustomElementRegistry();
  scopedDocRegistry.initialize(contentDocument);
  assert_equals(contentDocument.customElementRegistry, scopedDocRegistry);

  const element = document.createElement("div", { customElementRegistry: null });
  const clone = contentDocument.importNode(element);
  assert_equals(clone.customElementRegistry, scopedDocRegistry);
}, "Cloning with null registry (scoped registry target)");

test(t => {
  const contentDocument = document.implementation.createHTMLDocument();
  const scopedDocRegistry = new CustomElementRegistry();
  scopedDocRegistry.initialize(contentDocument);
  assert_equals(contentDocument.customElementRegistry, scopedDocRegistry);

  const scoped = new CustomElementRegistry();
  const element = document.createElement("div", { customElementRegistry: scoped });
  const clone = contentDocument.importNode(element);
  assert_equals(clone.customElementRegistry, scoped);
}, "Cloning with scoped registry (scoped registry target)");

test(t => {
  const contentDocument = document.implementation.createHTMLDocument();
  const scopedDocRegistry = new CustomElementRegistry();
  scopedDocRegistry.initialize(contentDocument);
  assert_equals(contentDocument.customElementRegistry, scopedDocRegistry);

  const element = document.createElement("div");
  const elementShadow = element.attachShadow({ mode: "open", clonable: true, customElementRegistry: customElements });
  const clone = contentDocument.importNode(element);
  assert_equals(clone.shadowRoot.customElementRegistry, null);
}, "Cloning including shadow tree with explicit global registry (scoped registry target)");

// https://dom.spec.whatwg.org/#concept-node-adopt
// https://html.spec.whatwg.org/multipage/custom-elements.html#dom-customelementregistry-initialize

// These elements have the global registry while their shadow root has a scoped or null registry,
// or got it from initialize(). Once adopted, they have their new document's effective global
// custom element registry.

function createWindow(t) {
  const iframe = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => iframe.remove());
  return iframe.contentWindow;
}

function appendToShadowRoot(element, customElementRegistry) {
  const host = document.createElement("div");
  host.attachShadow({ mode: "open", customElementRegistry }).append(element);
  return host;
}

const targets = [
  ["a document with a global registry", t => {
    const contentWindow = createWindow(t);
    return [contentWindow.document, contentWindow.customElements];
  }],
  ["a document without a browsing context", () => [document.implementation.createHTMLDocument(), null]],
];

for (const [targetDescription, createTarget] of targets) {
  for (const [registryDescription, createRegistry] of [
    ["a scoped registry", () => new CustomElementRegistry()],
    ["a null registry", () => null],
  ]) {
    test(t => {
      const [targetDocument, targetRegistry] = createTarget(t);
      const element = document.createElement("div");
      const host = appendToShadowRoot(element, createRegistry());
      assert_equals(element.customElementRegistry, customElements, "before adoption");

      targetDocument.adoptNode(host);
      assert_equals(element.customElementRegistry, targetRegistry, "after adoption");
    }, `Element with the global registry in a shadow root with ${registryDescription}, whose host is adopted into ${targetDescription}`);

    test(t => {
      const [targetDocument, targetRegistry] = createTarget(t);
      const element = document.createElement("div");
      appendToShadowRoot(element, createRegistry());
      assert_equals(element.customElementRegistry, customElements, "before adoption");

      targetDocument.adoptNode(element);
      assert_equals(element.customElementRegistry, targetRegistry, "after adoption");
    }, `Element with the global registry in a shadow root with ${registryDescription}, adopted into ${targetDescription}`);
  }

  test(t => {
    const [targetDocument, targetRegistry] = createTarget(t);
    const element = document.createElement("div", { customElementRegistry: null });
    const host = appendToShadowRoot(element, new CustomElementRegistry());
    customElements.initialize(element);
    assert_equals(element.customElementRegistry, customElements, "before adoption");

    targetDocument.adoptNode(host);
    assert_equals(element.customElementRegistry, targetRegistry, "after adoption");
  }, `Element given the global registry by initialize() in a shadow root with a scoped registry, whose host is adopted into ${targetDescription}`);

  test(t => {
    const [targetDocument, targetRegistry] = createTarget(t);
    const element = document.createElement("div", { customElementRegistry: null });
    customElements.initialize(element);
    assert_equals(element.customElementRegistry, customElements, "before adoption");

    targetDocument.adoptNode(element);
    assert_equals(element.customElementRegistry, targetRegistry, "after adoption");
  }, `Element without a parent given the global registry by initialize(), adopted into ${targetDescription}`);
}

test(t => {
  const contentWindow = createWindow(t);
  const element = document.createElement("test-element");
  const host = appendToShadowRoot(element, new CustomElementRegistry());
  contentWindow.document.body.append(host);
  assert_equals(element.customElementRegistry, contentWindow.customElements, "after appending");

  class TestElement extends contentWindow.HTMLElement {}
  contentWindow.customElements.define("test-element", TestElement);
  assert_true(element instanceof TestElement, "upgraded");
}, "Element with the global registry in a shadow root with a scoped registry, whose host is appended to another document, is upgraded by that document's registry");

test(t => {
  const contentWindow = createWindow(t);
  const contentDocument = contentWindow.document;
  const element = document.createElement("test-element");
  appendToShadowRoot(element, new CustomElementRegistry());
  const host = contentDocument.body.appendChild(contentDocument.createElement("div"));
  host.attachShadow({ mode: "open", customElementRegistry: new contentWindow.CustomElementRegistry() }).append(element);
  assert_equals(element.customElementRegistry, contentWindow.customElements, "after appending");

  class TestElement extends contentWindow.HTMLElement {}
  contentWindow.customElements.define("test-element", TestElement);
  assert_true(element instanceof TestElement, "upgraded");
}, "Element with the global registry, appended from a shadow root with a scoped registry to one in another document, is upgraded by that document's registry");

test(t => {
  const contentWindow = createWindow(t);
  const host = document.body.appendChild(document.createElement("div"));
  t.add_cleanup(() => host.remove());
  const shadow = host.attachShadow({ mode: "open", customElementRegistry: new CustomElementRegistry() });
  const element = shadow.appendChild(document.createElement("appended-back-element"));
  contentWindow.document.adoptNode(element);
  assert_equals(element.customElementRegistry, contentWindow.customElements, "after adoption");

  shadow.append(element);
  assert_equals(element.customElementRegistry, customElements, "after appending back");
  class AppendedBackElement extends HTMLElement {}
  customElements.define("appended-back-element", AppendedBackElement);
  assert_true(element instanceof AppendedBackElement, "upgraded");
}, "Element with the global registry, adopted into another document and appended back to its shadow root with a scoped registry, is upgraded by the global registry");

test(t => {
  const contentWindow = createWindow(t);
  const parent = document.createElement("div");
  const host = appendToShadowRoot(parent, new CustomElementRegistry());
  const child = parent.appendChild(document.createElement("div", { customElementRegistry: null }));
  assert_equals(child.customElementRegistry, null, "before adoption");

  contentWindow.document.adoptNode(host);
  assert_equals(child.customElementRegistry, contentWindow.customElements, "after adoption");
}, "Element with a null registry whose parent has the global registry in a shadow root with a scoped registry, whose host is adopted into a document with a global registry");

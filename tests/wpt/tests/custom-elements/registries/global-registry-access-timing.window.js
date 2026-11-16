// https://dom.spec.whatwg.org/#concept-attach-a-shadow-root
// https://dom.spec.whatwg.org/#concept-node-clone
// https://dom.spec.whatwg.org/#concept-node-adopt
// https://dom.spec.whatwg.org/#dom-document-importnode
// https://html.spec.whatwg.org/multipage/custom-elements.html#concept-try-upgrade

// Each test uses new windows, so that nothing accesses their customElements before the test does.
// Whether that happens before or after the nodes are created must not matter.

function createWindow(t, accessCustomElements) {
  const iframe = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => iframe.remove());
  if (accessCustomElements)
    iframe.contentWindow.customElements;
  return iframe.contentWindow;
}

function defineAndAppend(contentWindow, node) {
  class TestElement extends contentWindow.HTMLElement {}
  contentWindow.customElements.define("test-element", TestElement);
  contentWindow.document.body.append(node);
  return TestElement;
}

for (const accessed of [false, true]) {
  const when = accessed ? "after" : "before";

  test(t => {
    const contentWindow = createWindow(t, accessed);
    const host = contentWindow.document.createElement("div");
    const shadow = host.attachShadow({ mode: "open" });
    shadow.innerHTML = "<test-element></test-element>";
    const element = shadow.firstChild;

    const TestElement = defineAndAppend(contentWindow, host);
    assert_true(element instanceof TestElement, "upgraded");
    assert_equals(element.customElementRegistry, contentWindow.customElements, "element registry");
    assert_equals(shadow.customElementRegistry, contentWindow.customElements, "shadow root registry");
  }, `Shadow root attached ${when} customElements is accessed`);

  test(t => {
    const contentWindow = createWindow(t, accessed);
    const container = contentWindow.document.createElement("div");
    container.setHTMLUnsafe('<div><template shadowrootmode="open"><test-element></test-element></template></div>');
    const shadow = container.firstChild.shadowRoot;
    const element = shadow.firstChild;

    const TestElement = defineAndAppend(contentWindow, container);
    assert_true(element instanceof TestElement, "upgraded");
    assert_equals(element.customElementRegistry, contentWindow.customElements, "element registry");
    assert_equals(shadow.customElementRegistry, contentWindow.customElements, "shadow root registry");
  }, `Declarative shadow root attached ${when} customElements is accessed`);

  test(t => {
    const contentWindow = createWindow(t, accessed);
    const host = contentWindow.document.createElement("div");
    host.attachShadow({ mode: "open", clonable: true }).innerHTML = "<test-element></test-element>";
    const clone = host.cloneNode(true);
    const shadow = clone.shadowRoot;
    const element = shadow.firstChild;

    const TestElement = defineAndAppend(contentWindow, clone);
    assert_true(element instanceof TestElement, "upgraded");
    assert_equals(element.customElementRegistry, contentWindow.customElements, "element registry");
    assert_equals(shadow.customElementRegistry, contentWindow.customElements, "shadow root registry");
  }, `Shadow root cloned ${when} customElements is accessed`);

  test(t => {
    const contentWindow = createWindow(t, accessed);
    const host = document.createElement("div");
    host.attachShadow({ mode: "open", clonable: true }).innerHTML = "<test-element></test-element>";
    const clone = contentWindow.document.importNode(host, true);
    const shadow = clone.shadowRoot;
    const element = shadow.firstChild;

    const TestElement = defineAndAppend(contentWindow, clone);
    assert_true(element instanceof TestElement, "upgraded");
    assert_equals(element.customElementRegistry, contentWindow.customElements, "element registry");
    assert_equals(shadow.customElementRegistry, contentWindow.customElements, "shadow root registry");
  }, `Shadow root imported into a document ${when} its customElements is accessed`);

  test(t => {
    const contentWindow = createWindow(t, accessed);
    const template = contentWindow.document.createElement("template");
    template.innerHTML = "<test-element></test-element>";
    const clone = contentWindow.document.importNode(template.content, true);
    const element = clone.firstChild;

    const TestElement = defineAndAppend(contentWindow, clone);
    assert_true(element instanceof TestElement, "upgraded");
    assert_equals(element.customElementRegistry, contentWindow.customElements, "element registry");
  }, `Template contents imported ${when} customElements is accessed`);

  test(t => {
    const contentWindow = createWindow(t, accessed);
    const host = contentWindow.document.createElement("div");
    const shadow = host.attachShadow({ mode: "open", customElementRegistry: null });
    const element = contentWindow.document.createElement("test-element");
    shadow.append(element);

    const TestElement = defineAndAppend(contentWindow, host);
    assert_true(element instanceof TestElement, "upgraded");
    assert_equals(element.customElementRegistry, contentWindow.customElements, "element registry");
    assert_equals(shadow.customElementRegistry, null, "shadow root registry");
  }, `Element appended to a shadow root with a null registry ${when} customElements is accessed`);
}

for (const [oldAccessed, newAccessed, when] of [
  [false, false, "before either document's customElements is accessed"],
  [true, false, "after only the old document's customElements is accessed"],
  [false, true, "after only the new document's customElements is accessed"],
  [true, true, "after both documents' customElements are accessed"],
]) {
  test(t => {
    const oldWindow = createWindow(t, oldAccessed);
    const newWindow = createWindow(t, newAccessed);
    const host = oldWindow.document.createElement("div");
    const shadow = host.attachShadow({ mode: "open" });
    shadow.innerHTML = "<test-element></test-element>";
    const element = shadow.firstChild;
    newWindow.document.adoptNode(host);

    const TestElement = defineAndAppend(newWindow, host);
    assert_true(element instanceof TestElement, "upgraded");
    assert_equals(element.customElementRegistry, newWindow.customElements, "element registry");
    assert_equals(shadow.customElementRegistry, newWindow.customElements, "shadow root registry");
  }, `Shadow root adopted ${when}`);
}

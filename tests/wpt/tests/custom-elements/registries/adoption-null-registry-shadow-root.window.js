// https://dom.spec.whatwg.org/#concept-node-adopt
// https://dom.spec.whatwg.org/#concept-node-clone
// https://html.spec.whatwg.org/multipage/custom-elements.html#dom-customelementregistry-initialize

function createDocumentWithScopedRegistry() {
  const contentDocument = document.implementation.createHTMLDocument();
  new CustomElementRegistry().initialize(contentDocument);
  return contentDocument;
}

test(() => {
  const host = document.createElement("div");
  const shadow = host.attachShadow({ mode: "open", customElementRegistry: null });
  createDocumentWithScopedRegistry().body.append(host);
  assert_equals(shadow.customElementRegistry, null);
}, "Shadow root with a null registry adopted into a document with a scoped registry");

test(() => {
  const shadow = document.implementation.createHTMLDocument().createElement("div").attachShadow({ mode: "open" });
  assert_equals(shadow.customElementRegistry, null, "before adoption");
  createDocumentWithScopedRegistry().body.append(shadow.host);
  assert_equals(shadow.customElementRegistry, null, "after adoption");
}, "Shadow root attached in a document with a null registry, adopted into a document with a scoped registry");

test(t => {
  const host = document.createElement("div");
  const shadow = host.attachShadow({ mode: "open" });
  shadow.innerHTML = "<test-element></test-element>";
  const element = shadow.firstChild;
  document.implementation.createHTMLDocument().body.append(host);
  assert_equals(shadow.customElementRegistry, null, "shadow root registry in a document with a null registry");
  assert_equals(element.customElementRegistry, null, "element registry in a document with a null registry");

  document.body.append(host);
  t.add_cleanup(() => host.remove());
  assert_equals(shadow.customElementRegistry, customElements, "shadow root registry");
  assert_equals(element.customElementRegistry, customElements, "element registry");
}, "Shadow root adopted into a document with a null registry and back");

test(t => {
  const host = document.createElement("div");
  host.attachShadow({ mode: "open", clonable: true }).innerHTML = "<test-element></test-element>";
  const clone = document.implementation.createHTMLDocument().importNode(host, true);
  const shadow = clone.shadowRoot;
  const element = shadow.firstChild;
  assert_equals(shadow.customElementRegistry, null, "shadow root registry in a document with a null registry");
  assert_equals(element.customElementRegistry, null, "element registry in a document with a null registry");

  document.body.append(clone);
  t.add_cleanup(() => clone.remove());
  assert_equals(shadow.customElementRegistry, customElements, "shadow root registry");
  assert_equals(element.customElementRegistry, customElements, "element registry");
}, "Shadow root cloned into a document with a null registry, then adopted");

test(t => {
  const template = document.createElement("template");
  template.setHTMLUnsafe("<div><template shadowrootmode=open shadowrootclonable><test-element></test-element></template></div>");
  assert_equals(template.content.firstChild.shadowRoot.customElementRegistry, null, "shadow root registry in template contents");
  const clone = document.importNode(template.content, true);
  const host = clone.firstChild;
  document.body.append(clone);
  t.add_cleanup(() => host.remove());
  assert_equals(host.shadowRoot.customElementRegistry, null, "shadow root registry");
  assert_equals(host.shadowRoot.firstChild.customElementRegistry, null, "element registry");
}, "Shadow root with a null registry cloned into a document with a global registry");

test(() => {
  const shadow = document.implementation.createHTMLDocument().createElement("div").attachShadow({ mode: "open" });
  const registry = new CustomElementRegistry();
  registry.initialize(shadow);
  assert_equals(shadow.customElementRegistry, registry);
}, "initialize() on a shadow root attached in a document with a null registry");

test(() => {
  const host = document.createElement("div");
  const shadow = host.attachShadow({ mode: "open" });
  document.implementation.createHTMLDocument().body.append(host);
  const registry = new CustomElementRegistry();
  registry.initialize(shadow);
  assert_equals(shadow.customElementRegistry, registry);
}, "initialize() on a shadow root adopted into a document with a null registry");

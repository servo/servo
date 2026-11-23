// An element whose custom element registry is a scoped registry it got from its document or its
// shadow root keeps that registry when adopted, as adoption leaves scoped registries alone.
// https://dom.spec.whatwg.org/#concept-node-adopt

function createScopedRegistryDocument() {
  const doc = document.implementation.createHTMLDocument();
  const registry = new CustomElementRegistry();
  registry.initialize(doc);
  return [doc, registry];
}

function createWindowDocument(t) {
  const iframe = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => iframe.remove());
  return iframe.contentDocument;
}

const targets = [
  ["a document with a global registry", t => createWindowDocument(t)],
  ["a document with a scoped registry", () => createScopedRegistryDocument()[0]],
  ["a document without a registry", () => document.implementation.createHTMLDocument()],
];

for (const [targetDescription, createTarget] of targets) {
  test(t => {
    const [doc, registry] = createScopedRegistryDocument();
    const element = doc.createElement("a-b");
    assert_equals(element.customElementRegistry, registry);
    createTarget(t).adoptNode(element);
    assert_equals(element.customElementRegistry, registry);
  }, `Element of a document with a scoped registry, adopted into ${targetDescription}`);

  test(t => {
    const [doc, registry] = createScopedRegistryDocument();
    const element = doc.body.appendChild(doc.createElement("a-b"));
    assert_equals(element.customElementRegistry, registry);
    createTarget(t).adoptNode(doc.body);
    assert_equals(element.customElementRegistry, registry);
  }, `Element in a document with a scoped registry, whose parent is adopted into ${targetDescription}`);

  test(t => {
    const registry = new CustomElementRegistry();
    const host = document.createElement("div");
    const shadowRoot = host.attachShadow({ mode: "open", customElementRegistry: registry });
    shadowRoot.innerHTML = "<a-b></a-b>";
    const element = shadowRoot.firstChild;
    assert_equals(element.customElementRegistry, registry);
    createTarget(t).adoptNode(host);
    assert_equals(element.customElementRegistry, registry);
  }, `Element parsed into a shadow root with a scoped registry, whose host is adopted into ${targetDescription}`);

  test(t => {
    const registry = new CustomElementRegistry();
    const host = document.createElement("div");
    const shadowRoot = host.attachShadow({ mode: "open", customElementRegistry: registry });
    const element = shadowRoot.appendChild(document.createElement("a-b", { customElementRegistry: registry }));
    assert_equals(element.customElementRegistry, registry);
    createTarget(t).adoptNode(host);
    assert_equals(element.customElementRegistry, registry);
  }, `Element created with the scoped registry of the shadow root it is appended to, whose host is adopted into ${targetDescription}`);
}

test(t => {
  const [doc, registry] = createScopedRegistryDocument();
  const element = doc.createElement("a-b");
  const shadowRoot = createWindowDocument(t).createElement("div").attachShadow({ mode: "open", customElementRegistry: new CustomElementRegistry() });
  shadowRoot.appendChild(element);
  assert_equals(element.customElementRegistry, registry);
}, "Element of a document with a scoped registry, appended to a shadow root with another scoped registry in a document with a global registry");

test(() => {
  const [doc] = createScopedRegistryDocument();
  const element = document.createElement("a-b");
  assert_equals(element.customElementRegistry, customElements);
  const shadowRoot = doc.createElement("div").attachShadow({ mode: "open", customElementRegistry: new CustomElementRegistry() });
  shadowRoot.appendChild(element);
  assert_equals(element.customElementRegistry, null);
}, "Element with the global registry, appended to a shadow root with a scoped registry in a document with a scoped registry");

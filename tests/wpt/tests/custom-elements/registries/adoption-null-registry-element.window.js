// https://dom.spec.whatwg.org/#concept-node-adopt

test(t => {
  const iframe = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => iframe.remove());
  const contentDocument = iframe.contentDocument;
  const container = document.createElement("div");
  container.setHTMLUnsafe('<div><template shadowrootmode="open" shadowrootcustomelementregistry><test-element><test-element></test-element></test-element></template></div>');
  const outer = container.firstChild.shadowRoot.firstChild;
  const inner = outer.firstChild;
  contentDocument.body.append(container);
  contentDocument.body.append(outer, inner);
  assert_equals(outer.customElementRegistry, null, "element whose parent was the shadow root");
  assert_equals(inner.customElementRegistry, null, "element whose parent was an element");
}, "Null registry elements adopted in a shadow root with a null registry stay null when appended elsewhere");

test(t => {
  const host = document.body.appendChild(document.createElement("div"));
  t.add_cleanup(() => host.remove());
  const shadow = host.attachShadow({ mode: "open", customElementRegistry: new CustomElementRegistry() });
  const contentDocument = document.implementation.createHTMLDocument();
  const parent = contentDocument.createElement("div");
  const child = parent.appendChild(contentDocument.createElement("test-element"));
  shadow.append(parent);
  assert_equals(parent.customElementRegistry, customElements, "parent registry");
  assert_equals(child.customElementRegistry, customElements, "child registry");
}, "Null registry elements appended from a document with a null registry into a shadow root with a scoped registry");

test(t => {
  const iframe = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => iframe.remove());
  const shadow = document.createElement("div").attachShadow({ mode: "open", customElementRegistry: new CustomElementRegistry() });
  const element = shadow.appendChild(document.createElement("test-element", { customElementRegistry: null }));
  iframe.contentDocument.body.append(element);
  assert_equals(element.customElementRegistry, iframe.contentWindow.customElements);
}, "Null registry element appended from a shadow root with a scoped registry to another document");

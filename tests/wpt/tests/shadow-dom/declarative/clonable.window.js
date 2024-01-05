test(() => {
  const div = document.createElement("div");
  const root = div.attachShadow({ mode: "open", clonable: true });
  root.appendChild(document.createElement("input"));
  assert_true(root.clonable, "clonable attribute");

  const cloned = div.cloneNode(true);
  assert_equals(cloned.shadowRoot.children.length, 1, "children count");
  assert_equals(cloned.shadowRoot.children[0].localName, "input", "children content");
}, "attachShadow with clonable: true");

test(() => {
  const template = document.createElement("template");
  assert_false(template.hasAttribute("shadowrootcustomelements"));
  assert_equals(template.shadowRootCustomElements, "");

  template.shadowRootCustomElements = "blah";
  assert_equals(template.getAttribute("shadowrootcustomelements"), "blah");
  assert_equals(template.shadowRootCustomElements, "blah");
}, "shadowRootCustomElements reflects as string");

test(() => {
  const div = document.createElement("div");
  div.setHTMLUnsafe(`<div><template shadowrootmode=open shadowrootcustomelements shadowrootserializable></template></div>`);
  assert_equals(div.firstChild.firstChild, null);
  assert_equals(div.getHTML({ serializableShadowRoots: true }), "<div><template shadowrootmode=\"open\" shadowrootserializable=\"\" shadowrootcustomelements=\"\"></template></div>");
}, "Serializing a ShadowRoot with a null registry");

test(() => {
  const div = document.createElement("div");
  div.setHTMLUnsafe(`<div><template shadowrootmode=open shadowrootcustomelements shadowrootserializable></template></div>`);
  const registry = new CustomElementRegistry();
  registry.initialize(div.firstChild.shadowRoot);
  assert_equals(div.firstChild.shadowRoot.customElements, registry);
  assert_equals(div.getHTML({ serializableShadowRoots: true }), "<div><template shadowrootmode=\"open\" shadowrootserializable=\"\" shadowrootcustomelements=\"\"></template></div>");
}, "Serializing a ShadowRoot with a registry that differs from its host");

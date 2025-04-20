// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: global=window,dedicatedworker,shadowrealm-in-window

idl_test(
  ["scoped-custom-elements-registry.tentative"],
  ["html", "dom"],
  (idl_array) => {
    let element = document.createElement("div");
    let shadowRoot = element.attachShadow({ mode: "open" });
    let customElementRegistry = new CustomElementRegistry();
    let templateElement = document.createElement("template");
    idl_array.add_objects({
      document,
      element,
      shadowRoot,
      customElementRegistry,
      templateElement,
    });
  },
);

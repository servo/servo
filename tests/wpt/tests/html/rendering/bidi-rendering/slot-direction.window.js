// https://html.spec.whatwg.org/multipage/rendering.html#bidi-rendering
// https://github.com/whatwg/html/pull/9796
// https://github.com/whatwg/html/pull/9880

for (let t of [
  {
    description: "<slot> inherits direction from parent",
    shadow_tree: `
      <div dir=ltr data-expected="ltr">
        <slot data-expected="ltr"></slot>
      </div>
    `,
    host_dir: "rtl",
  },
  {
    description: "<slot> inherits CSS direction from parent",
    shadow_tree: `
      <div style="direction: ltr" data-expected="ltr">
        <slot data-expected="ltr"></slot>
      </div>
    `,
    host_dir: "rtl",
  },
  {
    description: "<slot dir=ltr>",
    shadow_tree: `
      <slot dir="ltr" data-expected="ltr"></slot>
    `,
    host_dir: "rtl",
  },
  {
    description: "<slot dir=rtl>",
    shadow_tree: `
      <slot dir="rtl" data-expected="rtl"></slot>
    `,
    host_dir: "ltr",
  },
  {
    description: "<slot dir=auto> resolving to LTR",
    shadow_tree: `
      <slot dir="ltr" data-expected="ltr"></slot>
    `,
    host_dir: "rtl",
    host_contents: "A",
  },
  {
    description: "<slot dir=auto> resolving to RTL",
    shadow_tree: `
      <slot dir="rtl" data-expected="rtl"></slot>
    `,
    host_dir: "ltr",
    host_contents: "\u0627",
  },
]) {
  test(() => {
    let host = document.createElement("div");
    document.body.appendChild(host);
    host.dir = t.host_dir;
    if ("host_contents" in t) {
      host.innerHTML = t.host_contents;
    }

    let root = host.attachShadow({mode: "open"});
    root.innerHTML = t.shadow_tree;

    for (let e of Array.from(root.querySelectorAll("[data-expected]"))) {
      assert_equals(getComputedStyle(e).direction, e.getAttribute("data-expected"), `direction of ${e.nodeName}`);
    }

    host.remove();
  }, `<slot> element sets CSS direction property: ${t.description}`);
}

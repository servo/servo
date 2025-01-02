const TESTS = [
  {
    title: "lang only on slot",
    light_tree: `
      <div id="host" data-expected="en-US"><span data-expected="en-US"></span></div>
    `,
    shadow_tree: `
      <slot lang="en-AU" data-expected="en-AU"></slot>
    `,
  },
  {
    title: "lang only on host",
    light_tree: `
      <div id="host" lang="en-AU" data-expected="en-AU"><span data-expected="en-AU"></span></div>
    `,
    shadow_tree: `
      <slot data-expected="en-AU"></slot>
    `,
  },
  {
    title: "lang on host and slot",
    light_tree: `
      <div id="host" lang="en-AU" data-expected="en-AU"><span data-expected="en-AU"></span></div>
    `,
    shadow_tree: `
      <slot lang="en-GB" data-expected="en-GB"></slot>
    `,
  },
  {
    title: "lang on host and slotted element",
    light_tree: `
      <div id="host" lang="en-AU" data-expected="en-AU"><span lang="en-GB" data-expected="en-GB"></span></div>
    `,
    shadow_tree: `
      <slot data-expected="en-AU"></slot>
    `,
  },
  {
    title: "lang on host and slot and slotted element",
    light_tree: `
      <div id="host" lang="en-AU" data-expected="en-AU"><span lang="en-GB" data-expected="en-GB"></span></div>
    `,
    shadow_tree: `
      <slot lang="en-NZ" data-expected="en-NZ"></slot>
    `,
  },
  {
    title: "lang on slot inherits from parent",
    light_tree: `
      <div id="host" lang="en-GB" data-expected="en-GB"><span lang="en-US" data-expected="en-US"></span></div>
    `,
    shadow_tree: `
      <div lang="en-CA" data-expected="en-CA">
        <slot data-expected="en-CA"></slot>
      </div>
    `,
  },
];

const container = document.createElement("div");
document.body.append(container);
container.lang = "en-US";

for (const obj of TESTS) {
  test(() => {
    container.innerHTML = obj.light_tree;
    let shadow = container.querySelector("#host").attachShadow({mode: "open"});
    shadow.innerHTML = obj.shadow_tree;
    for (const element of Array.from(container.querySelectorAll("[data-expected]")).concat(Array.from(shadow.querySelectorAll("[data-expected]")))) {
      const expected = element.getAttribute("data-expected");
      assert_true(element.matches(`:lang(${expected})`), `element matches expected language ${expected}`);
      for (const other_lang of ["en-US", "en-AU", "en-GB", "en-NZ", "en-CA"]) {
        if (expected != other_lang) {
          assert_false(element.matches(`:lang(${other_lang})`), `element does not match language ${other_lang}`);
        }
      }
    }
  }, obj.title);
}

container.remove();

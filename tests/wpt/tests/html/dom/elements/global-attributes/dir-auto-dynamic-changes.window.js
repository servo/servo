function setup_tree(light_tree, shadow_tree) {
  let body = document.body;
  let old_length = body.childNodes.length;
  body.insertAdjacentHTML("beforeend", light_tree.trim());
  if (body.childNodes.length != old_length + 1) {
    throw "unexpected markup";
  }
  let result = body.lastChild;
  if (shadow_tree) {
    let shadow = result.querySelector("#root").attachShadow({mode: "open"});
    shadow.innerHTML = shadow_tree.trim();
    return [result, shadow];
  }
  return result;
}

test(t => {
  let a = setup_tree(`
    <div id="a" dir="auto">
      <div id="b"></div>
      hello
    </div>
  `);

  let acs = getComputedStyle(a);
  assert_true(a.matches(":dir(ltr)"), ":dir(ltr) matches before insertion");
  assert_false(a.matches(":dir(rtl)"), ":dir(rtl) does not match before insertion");
  assert_equals(acs.direction, "ltr", "CSSdirection before insertion");
  b.innerHTML = "\u05D0";
  assert_false(a.matches(":dir(ltr)"), ":dir(ltr) does not match after insertion");
  assert_true(a.matches(":dir(rtl)"), ":dir(rtl) matches after insertion");
  assert_equals(acs.direction, "rtl", "CSSdirection after insertion");

  a.remove();
}, "dynamic insertion of RTL text in a child element");

test(() => {
  let div_rtlchar = document.createElement("div");
  div_rtlchar.innerHTML = "\u05D0";

  let container1 = document.createElement("div");
  document.body.appendChild(container1);
  let container2 = document.createElement("div");

  for (let container of [container1, container2]) {
    container.dir = "auto";
    assert_true(container.matches(":dir(ltr)"));
    container.appendChild(div_rtlchar);
    assert_false(container.matches(":dir(ltr)"));
    div_rtlchar.remove();
    assert_true(container.matches(":dir(ltr)"));
  }

  container1.remove();
}, "dir=auto changes for content insertion and removal, in and out of document");

test(() => {
  let tree, shadow;
  [tree, shadow] = setup_tree(`
    <div>
      <div id="root">
        <span id="l">A</span>
        <span id="r">\u05D0</span>
      </div>
    </div>
  `, `
    <slot id="one" name="one" dir="auto">\u05D0</slot>
    <slot id="two" dir="auto"></slot>
  `);

  let one = shadow.getElementById("one");
  let two = shadow.getElementById("two");
  let l = tree.querySelector("#l");
  let r = tree.querySelector("#r");
  assert_false(one.matches(":dir(ltr)"), "#one while empty");
  assert_true(two.matches(":dir(ltr)"), "#two with both spans");
  l.slot = "one";
  assert_true(one.matches(":dir(ltr)"), "#one with LTR child span");
  assert_false(two.matches(":dir(ltr)"), "#two with RTL child span");
  r.slot = "one";
  assert_true(one.matches(":dir(ltr)"), "#one with both child spans");
  assert_true(two.matches(":dir(ltr)"), "#two while empty");
  l.slot = "";
  assert_false(one.matches(":dir(ltr)"), "#one with RTL child span");
  assert_true(two.matches(":dir(ltr)"), "#two with LTR child span");

  tree.remove();
}, "dir=auto changes for slot reassignment");

test(() => {
  let tree, shadow;
  [tree, shadow] = setup_tree(`
    <div dir=auto>
      <div id=root>
        <div id=text>A</div>
      </div>
    </div>
  `, `
    <div dir=ltr>
      <slot id=slot dir=auto></slot>
    </div>
  `);

  let text = tree.querySelector("#text");
  let slot = shadow.querySelector("#slot");

  assert_true(tree.matches(":dir(ltr)"), "node tree ancestor before first text change");
  assert_true(slot.matches(":dir(ltr)"), "slot before first text change");
  text.innerText = "\u05D0";
  assert_false(tree.matches(":dir(ltr)"), "node tree ancestor after first text change");
  assert_false(slot.matches(":dir(ltr)"), "slot after first text change");
  tree.dir = "rtl";
  assert_false(tree.matches(":dir(ltr)"), "node tree ancestor before second text change");
  assert_false(slot.matches(":dir(ltr)"), "slot before second text change");
  text.innerText = "A";
  assert_false(tree.matches(":dir(ltr)"), "node tree ancestor after second text change");
  assert_true(slot.matches(":dir(ltr)"), "slot after second text change");
  slot.dir = "ltr";
  assert_false(tree.matches(":dir(ltr)"), "node tree ancestor before third text change");
  assert_true(slot.matches(":dir(ltr)"), "slot before third text change");
  text.innerText = "\u05D0";
  assert_false(tree.matches(":dir(ltr)"), "node tree ancestor after third text change");
  assert_true(slot.matches(":dir(ltr)"), "slot after third text change");
  slot.dir = "auto";
  tree.dir = "auto";
  assert_false(tree.matches(":dir(ltr)"), "node tree ancestor after fourth text change");
  assert_false(slot.matches(":dir(ltr)"), "slot after fourth text change");
  text.innerText = "A";
  assert_true(tree.matches(":dir(ltr)"), "node tree ancestor before fourth text change");
  assert_true(slot.matches(":dir(ltr)"), "slot before fourth text change");
  slot.dir = "rtl";
  assert_true(tree.matches(":dir(ltr)"), "node tree ancestor before fifth text change");
  assert_false(slot.matches(":dir(ltr)"), "slot before fifth text change");
  text.innerText = "\u05D0";
  assert_false(tree.matches(":dir(ltr)"), "node tree ancestor before fifth text change");
  assert_false(slot.matches(":dir(ltr)"), "slot before fifth text change");

  tree.remove();
}, "text changes affecting both slot and ancestor with dir=auto");

test(() => {
  let tree = setup_tree(`
    <div dir="auto">
      <span id="a1">A</span>
      <span id="aleph1">\u05D0</span>
      <span id="a2">A</span>
      <span id="aleph2">\u05D0</span>
    </div>
  `);

  let a1 = tree.querySelector("#a1");
  let aleph1 = tree.querySelector("#aleph1");
  assert_true(tree.matches(":dir(ltr)"), "initial state");
  assert_false(tree.matches(":dir(rtl)"), "initial state");
  a1.dir = "ltr";
  assert_false(tree.matches(":dir(ltr)"), "after change 1");
  a1.dir = "invalid";
  assert_true(tree.matches(":dir(ltr)"), "after change 2");
  a1.dir = "rtl";
  assert_false(tree.matches(":dir(ltr)"), "after change 3");
  a1.removeAttribute("dir");
  assert_true(tree.matches(":dir(ltr)"), "after change 4");
  a1.dir = "invalid";
  assert_true(tree.matches(":dir(ltr)"), "after change 5");
  a1.dir = "rtl";
  assert_false(tree.matches(":dir(ltr)"), "after change 6");
  aleph1.dir = "auto";
  assert_true(tree.matches(":dir(ltr)"), "after change 7");
  aleph1.dir = "invalid";
  assert_false(tree.matches(":dir(ltr)"), "after change 8");

  tree.remove();
}, "dynamic changes to subtrees excluded as a result of the dir attribute");

test(() => {
  let tree = setup_tree(`
    <div dir="auto">
      <!-- element goes here -->
    </div>
  `);

  let element = document.createElementNS("namespace", "element");
  let text = document.createTextNode("\u05D0");
  element.appendChild(text);
  tree.prepend(element);
  assert_not_equals(element.namespaceURI, tree.namespaceURI);

  assert_true(tree.matches(":dir(rtl)"), "initial state");
  assert_false(tree.matches(":dir(ltr)"), "initial state");
  text.data = "A";
  assert_true(tree.matches(":dir(ltr)"), "after dynamic change");
  assert_false(tree.matches(":dir(rtl)"), "after dynamic change");

  tree.remove();
}, "dynamic changes inside of non-HTML elements");

test(() => {
  let tree, shadow;
  [tree, shadow] = setup_tree(`
    <div dir="auto">
      <div id="root">
        <element xmlns="namespace">A</element>
        \u05D0
      </div>
    </div>
  `, `
    <div dir="ltr">
      <slot dir="auto">\u05D0</slot>
    </div>
  `);

  let element = tree.querySelector("element");
  let slot = shadow.querySelector("slot");
  let text = element.firstChild;

  assert_true(tree.matches(":dir(ltr)"), "initial state (tree)");
  assert_true(element.matches(":dir(ltr)"), "initial state (element)");
  assert_true(slot.matches(":dir(ltr)"), "initial state (slot)");

  text.data = "\u05D0";

  assert_true(tree.matches(":dir(rtl)"), "state after first change (tree)");
  assert_true(element.matches(":dir(rtl)"), "state after first change (element)");
  assert_true(slot.matches(":dir(rtl)"), "state after first change (slot)");

  text.data = "";

  assert_true(tree.matches(":dir(rtl)"), "state after second change (tree)");
  assert_true(element.matches(":dir(rtl)"), "state after second change (element)");
  assert_true(slot.matches(":dir(rtl)"), "state after second change (slot)");

  tree.remove();
}, "slotted non-HTML elements");

test(() => {
  let tree, shadow;
  [tree, shadow] = setup_tree(`
    <div>
      <div id="root">
        <!-- element goes here -->
        \u05D0
      </div>
    </div>
  `, `
    <div dir="ltr">
      <slot></slot>
    </div>
  `);

  let element = document.createElementNS("namespace", "element");
  let text = document.createTextNode("A");
  element.appendChild(text);
  tree.querySelector("#root").prepend(element);

  assert_not_equals(element.namespaceURI, tree.namespaceURI);

  assert_true(tree.matches(":dir(ltr)"), "initial state (tree)");
  assert_true(element.matches(":dir(ltr)"), "initial state (element)");

  tree.dir = "auto";

  assert_true(tree.matches(":dir(ltr)"), "state after making dir=auto (tree)");
  assert_true(element.matches(":dir(ltr)"), "state after making dir=auto (element)");

  text.data = "\u05D0";

  assert_true(tree.matches(":dir(rtl)"), "state after first change (tree)");
  assert_true(element.matches(":dir(rtl)"), "state after first change (element)");

  text.data = "";

  assert_true(tree.matches(":dir(rtl)"), "state after second change (tree)");
  assert_true(element.matches(":dir(rtl)"), "state after second change (element)");

  tree.remove();
}, "slotted non-HTML elements after dynamically assigning dir=auto, and dir attribute ignored on non-HTML elements");

// META: script=dir-shadow-utils.js

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
  let [tree, shadow] = setup_tree(`
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
  let [tree, shadow] = setup_tree(`
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
  let [tree, shadow] = setup_tree(`
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
  let [tree, shadow] = setup_tree(`
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

test(() => {
  let e1 = setup_tree(`
    <div dir=auto>
      <div dir=ltr>
        \u05D0
      </div>
    </div>
  `);
  let e2 = e1.firstElementChild;
  assert_true(e1.matches(":dir(ltr)"), "parent is LTR before changes");
  assert_true(e2.matches(":dir(ltr)"), "child is LTR before changes");
  e2.removeAttribute("dir");
  assert_false(e1.matches(":dir(ltr)"), "parent is RTL after removing dir from child");
  assert_false(e2.matches(":dir(ltr)"), "child is RTL after removing dir from child");
}, "dir=auto ancestor considers text in subtree after removing dir=ltr from it");

test(() => {
  let tree1, shadow1;
  [tree1, shadow1] = setup_tree(`
    <div>
      <div id="root" dir="auto">
        <div id="root2">
          <span>A</span>
        </div>
      </div>
    </div>
  `,`
    <slot dir="auto"></slot>
  `);
  let tree2 = tree1.querySelector("#root2");
  let shadow2 = tree2.attachShadow({mode: 'open'});
  shadow2.innerHTML = '<slot dir="auto"></slot>';

  let slot1 = shadow1.querySelector("slot");
  let slot2 = shadow2.querySelector("slot");
  let span = tree1.querySelector("span");

  // span slotted in slot2 hosted in root2 slotted in slot1
  // span thus impacts auto-dir of two slots
  assert_true(slot1.matches(":dir(ltr)", "outer slot initially ltr"));
  assert_true(slot2.matches(":dir(ltr)", "inner slot initially ltr"));
  span.innerHTML = "\u05D0";
  assert_true(slot1.matches(":dir(rtl)", "outer slot changed to rtl"));
  assert_true(slot2.matches(":dir(rtl)", "inner slot changed to rtl"));

  tree1.remove();
}, 'Slotted content affects multiple dir=auto slots');

test(() => {
  let [tree, shadow] = setup_tree(`
    <div>
      <div id="root">
        <span>اختبر</span>
      </div>
    </div>
  `, `
    <slot dir="auto"></slot>
  `);

  let slot = shadow.querySelector("slot");
  assert_equals(html_direction(slot), "rtl", "slot initially rtl");
  let span = tree.querySelector("span");
  span.remove();
  assert_equals(html_direction(slot), "ltr", "slot is reset to ltr");
  tree.remove();
}, 'Removing slotted content resets direction on dir=auto slot');

test(() => {
  let [tree, shadow] = setup_tree(`
    <div>
      <div id=root>
        <div>
          <span>اختبر</span>
        </div>
      </div>
    </div>
  `,`
    <slot dir=auto></slot>
  `);

  let slot = shadow.querySelector("slot");
  assert_equals(html_direction(slot), "rtl", "slot initially rtl");
  let span = tree.querySelector("span");
  span.remove();
  assert_equals(html_direction(slot), "ltr", "slot is reset to ltr");
  tree.remove();
}, 'Removing child of slotted content changes direction on dir=auto slot');

test(() => {
  let tree;
  tree = setup_tree(`
    <div>
      <span>اختبر</span>
      <p>Text</p>
    </div>
  `);
  let p = tree.querySelector("p");
  assert_true(p.matches(":dir(ltr)"), "child initially ltr");
  tree.dir = "auto";
  assert_true(p.matches(":dir(rtl)"), "child updated to rtl");
  tree.remove();
}, 'Child directionality gets updated when dir=auto is set on parent');

test(() => {
  let [tree, shadow] = setup_tree(`
    <div>
      <div id=root>
        <input value="اختبر">
      </div>
    </div>
  `,`
    <slot dir=auto></slot>
  `);
  let slot = shadow.querySelector("slot");
  assert_equals(html_direction(slot), "ltr");
  tree.remove();
}, 'dir=auto slot is not affected by text in value of input element children');

test(() => {
  let tree = setup_tree(`
    <div>
      <input dir="auto" value="اختبر">
    </div>
  `);
  let inp = tree.querySelector("input");
  assert_equals(html_direction(inp), "rtl");
  inp.type = "month";
  assert_equals(html_direction(inp), "ltr");
  tree.remove();
}, 'input direction changes if it stops being auto-directionality form-associated');

test(() => {
  let [tree, shadow] = setup_tree(`
    <div>
      <div id=root dir=ltr>
        <span>اختبر</span>
      </div>
    </div>
  `,`
    <div dir=auto id=container>
      <slot></slot>
    </div>
  `);
  let div = shadow.querySelector("#container");
  let host = tree.querySelector("#root");
  assert_equals(html_direction(div), 'ltr', 'ltr inherited from host despite rtl content');
  // set rtl on host directly, test it is propagated through slots
  host.dir = "rtl";
  assert_equals(html_direction(div), 'rtl', 'host dir change propagated via slot');
  host.dir = "";
  assert_equals(html_direction(host), 'ltr', 'host dir reset to ltr');
  assert_equals(html_direction(div), 'ltr', 'host dir change propagated via slot');
  // host inherits rtl from parent, test it is still propagated through slots
  tree.dir = "rtl";
  assert_equals(html_direction(host), 'rtl', 'host inherited rtl from parent');
  assert_equals(html_direction(div), 'rtl', 'host dir change propagated via slot');
  tree.remove();
}, 'slot provides updated directionality from host to a dir=auto container');

test(() => {
  let input = setup_tree(`<input type="text">`);
  assert_equals(html_direction(input), "ltr", "initial direction of input");
  input.value = "\u05D0";
  assert_equals(html_direction(input), "ltr", "direction of input with RTL contents");
  input.dir = "auto";
  assert_equals(html_direction(input), "rtl", "direction of input dir=auto with RTL contents");
  input.value = "a";
  assert_equals(html_direction(input), "ltr", "direction of input dir=auto with LTR contents");
  input.dir = "rtl";
  assert_equals(html_direction(input), "rtl", "direction of input dir=rtl with LTR contents");
  input.value = "ab";
  assert_equals(html_direction(input), "rtl", "direction of input dir=rtl with LTR contents (2)");
  input.value = "\u05D0";
  assert_equals(html_direction(input), "rtl", "direction of input dir=rtl with RTL contents");

  let textarea = setup_tree(`<textarea dir="auto"></textarea>`);
  assert_equals(html_direction(textarea), "ltr", "direction of textarea dir=auto with empty contents");
  textarea.value = "a";
  assert_equals(html_direction(textarea), "ltr", "direction of textarea dir=auto with LTR contents");
  textarea.value = "\u05D0";
  assert_equals(html_direction(textarea), "rtl", "direction of textarea dir=auto with RTL contents");
  textarea.dir = "rtl";
  assert_equals(html_direction(textarea), "rtl", "direction of textarea dir=rtl with RTL contents");
  textarea.value = "a";
  assert_equals(html_direction(textarea), "rtl", "direction of textarea dir=rtl with LTR contents");
}, 'text input and textarea value changes should only be reflected in :dir() when dir=auto (value changes)');

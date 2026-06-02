test(() => {
  assert_true(document.documentElement.matches(":dir(ltr)"));
}, "Root element has a direction");

test(() => {
  const ele = document.createElement("foobar");
  assert_true(ele.matches(":dir(ltr)"));
}, "Element outside the document tree has a direction");

test(() => {
  const ele = document.createElementNS("foobar", "foobar");
  assert_true(ele.matches(":dir(ltr)"));
}, "Non-HTML element outside the document tree has a direction");

test(() => {
  const ele = document.createElement("foobar");
  ele.dir = "rtl";
  const ele2 = document.createElement("foobar");
  ele.append(ele2);
  assert_true(ele2.matches(":dir(rtl)"));
  ele.dir = "ltr";
  assert_true(ele2.matches(":dir(ltr)"), "direction after dynamic change");
}, "Element without direction has parent element direction");

test(() => {
  const ele = document.createElement("foobar");
  ele.dir = "rtl";
  const ele2 = document.createElementNS("foobar", "foobar");
  ele.append(ele2);
  assert_true(ele2.matches(":dir(rtl)"));
  ele.dir = "ltr";
  assert_true(ele2.matches(":dir(ltr)"), "direction after dynamic change");
}, "Non-HTML element without direction has parent element direction");

test(() => {
  let container1 = document.createElement("div");
  document.body.appendChild(container1);
  let container2 = document.createElement("div");

  for (let container of [container1, container2]) {
    container.dir = "rtl";
    let e = document.createElement("div");
    assert_true(e.matches(":dir(ltr)"));
    container.appendChild(e);
    assert_false(e.matches(":dir(ltr)"));
    e.remove();
    assert_true(e.matches(":dir(ltr)"));
  }

  container1.remove();
}, "dir inheritance is correct after insertion and removal from document");

test(() => {
  const ele = document.createElement("foobar");
  ele.dir = "auto";
  const ele2 = document.createElementNS("foobar", "foobar");
  ele.append(ele2);
  const text = document.createTextNode("\u05D0\u05D1\u05D2");
  ele2.append(text);
  assert_true(ele.matches(":dir(rtl)"), "is RTL before change");
  assert_true(ele2.matches(":dir(rtl)"), "child is RTL before change");
  text.data = "ABC";
  assert_true(ele.matches(":dir(ltr)"), "is LTR after change");
  assert_true(ele2.matches(":dir(ltr)"), "child is LTR after change");
}, "Non-HTML element text contents influence dir=auto");


for (const tag of ["style", "script"]) {
  test(() => {
    const e1 = document.createElement("div");
    e1.dir = "auto";

    const e2 = document.createElement(tag);
    const node = document.createTextNode("\u05D0");
    e2.appendChild(node);
    e1.appendChild(e2);
    assert_true(e1.matches(":dir(ltr)", "is LTR before change"));
    node.data = "ABC";
    assert_true(e1.matches(":dir(ltr)", "is LTR after change"));

  }, `${tag} element text contents do not influence dir=auto`);
}

for (const tag of ["style", "script", "input", "textarea"]) {
  test(() => {
    const e1 = document.createElement("div");
    e1.dir = "auto";
    const svg = document.createElement("svg");
    const e2 = document.createElementNS("http://www.w3.org/2000/svg", tag);
    const node = document.createTextNode("\u05D0");
    e2.appendChild(node);
    svg.appendChild(e2);
    e1.appendChild(svg);
    assert_true(e1.matches(":dir(rtl)", "is RTL before change"));
    node.data = "ABC";
    assert_true(e1.matches(":dir(ltr)", "is LTR after change"));
  }, `non-html ${tag} element text contents influence dir=auto`);
}

for (const dir of ["auto", "ltr"]) {
  test(() => {
    const e1 = document.createElement("div");
    e1.dir = "auto";
    const e2 = document.createElement("div");
    e2.dir = dir;
    e2.innerText = "A";
    e1.append(e2);
    assert_true(e1.matches(":dir(ltr)"), "parent is LTR before changes");
    assert_true(e2.matches(":dir(ltr)"), "child is LTR before changes");
    e2.removeAttribute("dir");
    assert_true(e1.matches(":dir(ltr)"), "parent is LTR after removing dir attribute on child");
    assert_true(e2.matches(":dir(ltr)"), "child is LTR after removing dir attribute on child");
    e2.firstChild.data = "\u05D0";
    assert_false(e1.matches(":dir(ltr)"), "parent is RTL after changing text in child");
    assert_false(e2.matches(":dir(ltr)"), "child is RTL after changing text in child");
  }, `text changes apply to dir=auto on further ancestor after removing dir=${dir} from closer ancestor`);
}

for (const bdi_test of [
  { markup: "<bdi dir=ltr>A</bdi>", expected: "ltr", desc: "dir=ltr with LTR contents" },
  { markup: "<bdi dir=ltr>\u05d0</bdi>", expected: "ltr", desc: "dir=ltr with RTL contents" },
  { markup: "<bdi dir=ltr></bdi>", expected: "ltr", desc: "dir=ltr empty" },
  { markup: "<bdi dir=rtl>A</bdi>", expected: "rtl", desc: "dir=rtl with LTR contents" },
  { markup: "<bdi dir=rtl>\u05d0</bdi>", expected: "rtl", desc: "dir=rtl with RTL contents" },
  { markup: "<bdi dir=rtl></bdi>", expected: "rtl", desc: "dir=rtl empty" },
  { markup: "<bdi dir=auto>A</bdi>", expected: "ltr", desc: "dir=auto with LTR contents" },
  { markup: "<bdi dir=auto>\u05d0</bdi>", expected: "rtl", desc: "dir=auto with RTL contents" },
  { markup: "<bdi dir=auto></bdi>", expected: "ltr", desc: "dir=auto empty" },
  { markup: "<bdi dir=auto>123</bdi>", expected: "ltr", desc: "dir=auto numbers" },
  { markup: "<bdi>A</bdi>", expected: "ltr", desc: "no dir attribute with LTR contents" },
  { markup: "<bdi>\u05d0</bdi>", expected: "rtl", desc: "no dir attribute with RTL contents" },
  { markup: "<bdi></bdi>", expected: "ltr", desc: "no dir attribute empty" },
]) {
  for (const parent_dir of [ "ltr", "rtl" ]) {
    test(() => {
      const parent_element = document.createElement("div");
      parent_element.dir = parent_dir;
      document.body.appendChild(parent_element);
      parent_element.innerHTML = bdi_test.markup;
      const bdi_element = parent_element.querySelector("bdi");
      let expected = bdi_test.expected;
      if (expected == "parent") {
        expected = parent_dir;
      }
      const not_expected = (expected == "ltr") ? "rtl" : "ltr";
      assert_true(bdi_element.matches(`:dir(${expected})`));
      assert_false(bdi_element.matches(`:dir(${not_expected})`));
      parent_element.remove();
    }, `directionality of bdi elements: ${bdi_test.desc} in ${parent_dir} parent`);
  }
}

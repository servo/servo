"use strict";

// Regression test for https://github.com/jsdom/jsdom/issues/3416
test(() => {
  const doc = (new DOMParser()).parseFromString(`<Foo><bar>baz</bar></Foo>`, "application/xml");

  assert_equals(doc.querySelector("Foo"), doc.documentElement, "Foo");
  assert_equals(doc.querySelector("bar"), doc.documentElement.firstChild, "bar");
  assert_equals(doc.querySelector("Foo bar"), doc.documentElement.firstChild, "Foo bar");
}, "Descendant combinator");

// Regression test for https://github.com/jsdom/jsdom/issues/3428
test(() => {
  const xml = `
<root>
  <aB>
    <c></c>
  </aB>
  <cd>
    <e></e>
  </cd>
</root>
`;
  const doc = (new DOMParser()).parseFromString(xml, "application/xml");

  assert_not_equals(doc.querySelector("c"), null, "c");
  assert_equals(doc.querySelector("aB *"), doc.querySelector("c"), "aB *");

  assert_not_equals(doc.querySelector("e"), null, "e");
  assert_equals(doc.querySelector("cd *"), doc.querySelector("e"), "cd *");
}, "Descendant combinator with *");

// Regression test for https://github.com/jsdom/jsdom/issues/3392#issuecomment-1180363238
test(() => {
  const doc = (new DOMParser()).parseFromString(`<elem>
    <Test>
        content
        <my-tag>abc</my-tag>
    </Test>
</elem>`, "text/xml");

  assert_not_equals(doc.querySelector("Test"), null, "Test");
  assert_equals(doc.querySelector("test"), null, "test");
  assert_not_equals(doc.querySelector("my-tag"), null, "my-tag");
  assert_not_equals(doc.querySelector("Test>my-tag"), null, "my-tag");
}, "Child combinator");

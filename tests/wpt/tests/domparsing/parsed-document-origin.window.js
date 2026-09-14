// META: title=Documents created by the parsing APIs inherit the creating document's origin

// https://html.spec.whatwg.org/multipage/dynamic-markup-insertion.html#dom-domparser-parsefromstring
// https://html.spec.whatwg.org/multipage/dynamic-markup-insertion.html#dom-parsehtml
// https://html.spec.whatwg.org/multipage/dynamic-markup-insertion.html#dom-parsehtmlunsafe
// https://dom.spec.whatwg.org/#dom-domimplementation-createdocument
// https://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument

test(() => {
  assert_not_equals(document.domain, "");
}, "This document has a tuple origin (prerequisite for the tests below)");

test(() => {
  const doc = new DOMParser().parseFromString("", "text/html");
  assert_equals(doc.domain, document.domain);
}, "DOMParser's parseFromString() with text/html");

test(() => {
  const doc = new DOMParser().parseFromString("<x/>", "text/xml");
  assert_equals(doc.domain, document.domain);
}, "DOMParser's parseFromString() with text/xml");

test(() => {
  const doc = Document.parseHTML("");
  assert_equals(doc.domain, document.domain);
}, "Document.parseHTML()");

test(() => {
  const doc = Document.parseHTMLUnsafe("");
  assert_equals(doc.domain, document.domain);
}, "Document.parseHTMLUnsafe()");

test(() => {
  const doc = document.implementation.createDocument(null, "");
  assert_equals(doc.domain, document.domain);
}, "createDocument()");

test(() => {
  const doc = document.implementation.createHTMLDocument("");
  assert_equals(doc.domain, document.domain);
}, "createHTMLDocument()");

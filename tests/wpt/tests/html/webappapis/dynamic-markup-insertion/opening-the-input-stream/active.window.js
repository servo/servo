function assertOpenIsEffective(doc, initialNodeCount) {
  assert_equals(doc.childNodes.length, initialNodeCount);

  // Test direct document.open() call.
  assert_equals(doc.open(), doc);
  assert_equals(doc.childNodes.length, 0, "after open: no nodes in document");
  doc.write("<!DOCTYPE html>");
  assert_equals(doc.childNodes.length, 1, "after write: doctype node in document");
  doc.close();
  assert_equals(doc.childNodes.length, 2, "after parser close: doctype node and an html element in document");

  // Test implicit document.open() call through write(). Since we called
  // doc.close() above, which sets the insertion point of the parser to
  // undefined, document.write() will run the document open steps.
  doc.write();
  assert_equals(doc.childNodes.length, 0, "after implicit open: no nodes in document");
  doc.write("<!DOCTYPE html>");
  assert_equals(doc.childNodes.length, 1, "after write: doctype node in document");
  doc.close();
  assert_equals(doc.childNodes.length, 2, "after parser close: doctype node and an html element in document");
}

test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  assertOpenIsEffective(frame.contentDocument, 1);
}, "document.open() removes the document's children (fully active document)");

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    const childFrame = frame.contentDocument.querySelector("iframe");
    const childDoc = childFrame.contentDocument;
    const childWin = childFrame.contentWindow;

    // Right now childDoc is still fully active.

    frame.onload = t.step_func_done(() => {
      // Now childDoc is still active but no longer fully active.
      assertOpenIsEffective(childDoc, 1);
    });
    frame.src = "/common/blank.html";
  });
  frame.src = "resources/page-with-frame.html";
}, "document.open() removes the document's children (active but not fully active document)");

test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  const doc = frame.contentDocument;

  // Right now the frame is connected and it has an active document.
  frame.remove();

  // Now the frame is no longer connected. Its document is no longer active.
  assertOpenIsEffective(doc, 1);
}, "document.open() removes the document's children (non-active document with an associated Window object; frame is removed)");

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.src = "resources/dummy.html";

  frame.onload = t.step_func(() => {
    const firstDocument = frame.contentDocument;
    // Right now the frame is connected and it has an active document.

    frame.onload = t.step_func_done(() => {
      // Now even though the frame is still connected, its document is no
      // longer active.
      assert_not_equals(frame.contentDocument, firstDocument);
      assertOpenIsEffective(firstDocument, 2);
    });

    frame.src = "/common/blank.html";
  });
}, "document.open() removes the document's children (non-active document with an associated Window object; navigated away)");

test(t => {
  const doc = document.implementation.createHTMLDocument();
  assertOpenIsEffective(doc, 2);
}, "document.open() removes the document's children (non-active document without an associated Window object; createHTMLDocument)");

test(t => {
  const doc = new DOMParser().parseFromString("", "text/html");
  assertOpenIsEffective(doc, 1);
}, "document.open() removes the document's children (non-active document without an associated Window object; DOMParser)");

async_test(t => {
  const xhr = new XMLHttpRequest();
  xhr.onload = t.step_func_done(() => {
    assert_equals(xhr.status, 200);
    assertOpenIsEffective(xhr.responseXML, 2);
  });
  xhr.responseType = "document";
  xhr.open("GET", "resources/dummy.html");
  xhr.send();
}, "document.open() removes the document's children (non-active document without an associated Window object; XMLHttpRequest)");

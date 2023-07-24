// Many of the active-related test cases in this file came from
// active.window.js. However, we cannot test the "navigated away" non-active
// case right now due to https://github.com/whatwg/html/issues/3997.

test(t => {
  const frame = document.body.appendChild(document.createElement("iframe")),
        body = frame.contentDocument.body;
  t.add_cleanup(() => frame.remove());
  const div = body.appendChild(frame.contentDocument.createElement("div"));
  div.addEventListener("click", t.unreached_func("element event listener not removed"));
  frame.contentDocument.open();
  div.click();
  frame.contentDocument.close();
}, "Standard event listeners are to be removed");

test(t => {
  const frame = document.body.appendChild(document.createElement("iframe")),
        body = frame.contentDocument.body;
  t.add_cleanup(() => frame.remove());
  frame.contentDocument.addEventListener("x", t.unreached_func("document event listener not removed"));
  body.addEventListener("x", t.unreached_func("body event listener not removed"));
  frame.contentDocument.open();
  frame.contentDocument.dispatchEvent(new Event("x"));
  body.dispatchEvent(new Event("x"));
  frame.contentDocument.close();
}, "Custom event listeners are to be removed");

test(t => {
  const frame = document.body.appendChild(document.createElement("iframe")),
        body = frame.contentDocument.body;
  t.add_cleanup(() => frame.remove());
  // Focus on the current window so that the frame's window is blurred.
  window.focus();
  assert_false(frame.contentDocument.hasFocus());
  frame.contentWindow.addEventListener("focus", t.unreached_func("window event listener not removed"));
  body.onfocus = t.unreached_func("body event listener not removed");
  frame.contentDocument.open();
  assert_equals(body.onfocus, null);
  frame.contentWindow.focus();
  frame.contentDocument.close();
}, "Standard event listeners are to be removed from Window");

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    const childFrame = frame.contentDocument.querySelector("iframe");
    const childWin = childFrame.contentWindow;
    const childDoc = childFrame.contentDocument;
    const childBody = childDoc.body;

    // Right now childDoc is still fully active.

    frame.onload = t.step_func_done(() => {
      // Focus on the current window so that the frame's window is blurred.
      window.focus();
      // Now childDoc is still active but no longer fully active.
      childWin.addEventListener("focus", t.unreached_func("window event listener not removed"));
      childBody.onfocus = t.unreached_func("body event listener not removed");

      childDoc.open();
      assert_equals(childBody.onfocus, null);

      // Now try to fire the focus event two different ways.
      childWin.focus();
      const focusEvent = new FocusEvent("focus");
      childWin.dispatchEvent(focusEvent);
      childDoc.close();
    });
    frame.src = "/common/blank.html";
  });
  frame.src = "resources/page-with-frame.html";
}, "Standard event listeners are to be removed from Window for an active but not fully active document");

test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  const win = frame.contentWindow;
  const doc = frame.contentDocument;
  const body = doc.body;

  // Right now the frame is connected and it has an active document.
  frame.remove();

  win.addEventListener("focus", t.unreached_func("window event listener not removed"));
  body.onfocus = t.unreached_func("body event listener not removed");
  doc.open();
  assert_equals(body.onfocus, null);

  // Now try to fire the focus event two different ways.
  win.focus();
  const focusEvent = new FocusEvent("focus");
  win.dispatchEvent(focusEvent);
  doc.close();
}, "Standard event listeners are to be removed from Window for a non-active document that is the associated Document of a Window (frame is removed)");

test(t => {
  let winHappened = 0;
  const winListener = t.step_func(() => { winHappened++; });
  window.addEventListener("focus", winListener);
  t.add_cleanup(() => { window.removeEventListener("focus", winListener); });

  let bodyHappened = 0;
  const bodyListener = t.step_func(() => { bodyHappened++; });
  document.body.onfocus = bodyListener;
  t.add_cleanup(() => { document.body.onfocus = null; });

  const doc = document.implementation.createHTMLDocument();
  doc.open();

  const focusEvent = new FocusEvent("focus");
  window.dispatchEvent(focusEvent);

  assert_equals(winHappened, 1);
  assert_equals(bodyHappened, 1);
}, "Standard event listeners are NOT to be removed from Window for a Window-less document (createHTMLDocument)");

test(t => {
  let winHappened = 0;
  const winListener = t.step_func(() => { winHappened++; });
  window.addEventListener("focus", winListener);
  t.add_cleanup(() => { window.removeEventListener("focus", winListener); });

  let bodyHappened = 0;
  const bodyListener = t.step_func(() => { bodyHappened++; });
  document.body.onfocus = bodyListener;
  t.add_cleanup(() => { document.body.onfocus = null; });

  const doc = new DOMParser().parseFromString("", "text/html");
  doc.open();

  const focusEvent = new FocusEvent("focus");
  window.dispatchEvent(focusEvent);

  assert_equals(winHappened, 1);
  assert_equals(bodyHappened, 1);
}, "Standard event listeners are NOT to be removed from Window for a Window-less document (DOMParser)");

async_test(t => {
  const xhr = new XMLHttpRequest();
  xhr.onload = t.step_func_done(() => {
    assert_equals(xhr.status, 200);
    const doc = xhr.responseXML;

    let winHappened = 0;
    const winListener = t.step_func(() => { winHappened++; });
    window.addEventListener("focus", winListener);
    t.add_cleanup(() => { window.removeEventListener("focus", winListener); });

    let bodyHappened = 0;
    const bodyListener = t.step_func(() => { bodyHappened++; });
    document.body.onfocus = bodyListener;
    t.add_cleanup(() => { document.body.onfocus = null; });

    doc.open();

    const focusEvent = new FocusEvent("focus");
    window.dispatchEvent(focusEvent);

    assert_equals(winHappened, 1);
    assert_equals(bodyHappened, 1);
  });
  xhr.responseType = "document";
  xhr.open("GET", "resources/dummy.html");
  xhr.send();
}, "Standard event listeners are NOT to be removed from Window for a Window-less document (XMLHttpRequest)");

test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.contentWindow.addEventListener("x", t.unreached_func("window event listener not removed"));
  frame.contentDocument.open();
  frame.contentWindow.dispatchEvent(new Event("x"));
  frame.contentDocument.close();
}, "Custom event listeners are to be removed from Window");

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
      childWin.addEventListener("x", t.unreached_func("window event listener not removed"));
      childDoc.open();
      childWin.dispatchEvent(new Event("x"));
      childDoc.close();
    });
    frame.src = "/common/blank.html";
  });
  frame.src = "resources/page-with-frame.html";
}, "Custom event listeners are to be removed from Window for an active but not fully active document");

test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  const win = frame.contentWindow;
  const doc = frame.contentDocument;

  // Right now the frame is connected and it has an active document.
  frame.remove();

  win.addEventListener("x", t.unreached_func("window event listener not removed"));
  doc.open();
  win.dispatchEvent(new Event("x"));
  doc.close();
}, "Custom event listeners are to be removed from Window for a non-active document that is the associated Document of a Window (frame is removed)");

test(t => {
  const doc = document.implementation.createHTMLDocument();
  let happened = false;
  window.addEventListener("createHTMLDocumentTest", t.step_func(() => { happened = true; }));
  doc.open();
  window.dispatchEvent(new Event("createHTMLDocumentTest"));
  assert_true(happened);
}, "Custom event listeners are NOT to be removed from Window for a Window-less document (createHTMLDocument)");

test(t => {
  const doc = new DOMParser().parseFromString("", "text/html");
  let happened = false;
  window.addEventListener("DOMParserTest", t.step_func(() => { happened = true; }));
  doc.open();
  window.dispatchEvent(new Event("DOMParserTest"));
  assert_true(happened);
}, "Custom event listeners are NOT to be removed from Window for a Window-less document (DOMParser)");

async_test(t => {
  const xhr = new XMLHttpRequest();
  xhr.onload = t.step_func_done(() => {
    assert_equals(xhr.status, 200);
    const doc = xhr.responseXML;
    let happened = false;
    window.addEventListener("XHRTest", t.step_func(() => { happened = true; }));
    doc.open();
    window.dispatchEvent(new Event("XHRTest"));
    assert_true(happened);
  });
  xhr.responseType = "document";
  xhr.open("GET", "resources/dummy.html");
  xhr.send();
}, "Custom event listeners are NOT to be removed from Window for a Window-less document (XMLHttpRequest)");

test(t => {
  const frame = document.body.appendChild(document.createElement("iframe")),
        body = frame.contentDocument.body;
  t.add_cleanup(() => frame.remove());
  const div = body.appendChild(frame.contentDocument.createElement("div"));
  div.onclick = t.unreached_func("element event listener not removed");
  frame.contentDocument.open();
  assert_equals(div.onclick, null);
  const e = frame.contentDocument.createEvent("mouseevents")
  e.initEvent("click", false, false);
  div.dispatchEvent(e);
  frame.contentDocument.close();
}, "IDL attribute event handlers are to be deactivated");

var thrower;

test(t => {
  const frame = document.body.appendChild(document.createElement("iframe")),
        body = frame.contentDocument.body;
  t.add_cleanup(() => frame.remove());
  const div = body.appendChild(frame.contentDocument.createElement("div"));
  thrower = t.step_func(() => { throw new Error('element event listener not removed'); });
  div.setAttribute("onclick", "parent.thrower()");
  assert_not_equals(div.onclick, null);
  frame.contentDocument.open();
  assert_equals(div.getAttribute("onclick"), "parent.thrower()");
  assert_equals(div.onclick, null);
  const e = frame.contentDocument.createEvent("mouseevents")
  e.initEvent("click", false, false);
  div.dispatchEvent(e);
  frame.contentDocument.close();
}, "Content attribute event handlers are to be deactivated");

test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  let once = false;
  frame.contentDocument.addEventListener("x", () => {
    frame.contentDocument.open();
    once = true;
  });
  frame.contentDocument.addEventListener("x", t.unreached_func("second event listener not removed"));
  frame.contentDocument.dispatchEvent(new Event("x"));
  assert_true(once);
  frame.contentDocument.close();
}, "Event listeners are to be removed with immediate effect");

test(t => {
  const frame = document.body.appendChild(document.createElement("iframe")),
        shadow = frame.contentDocument.body.attachShadow({ mode: "closed" }),
        shadowChild = shadow.appendChild(document.createElement("div")),
        shadowShadow = shadowChild.attachShadow({ mode: "open" }),
        nodes = [shadow, shadowChild, shadowShadow];
  t.add_cleanup(() => frame.remove());
  nodes.forEach(node => {
    node.addEventListener("x", t.unreached_func(node + "'s event listener not removed"));
  });
  frame.contentDocument.open();
  nodes.forEach(node => {
    node.dispatchEvent(new Event("x"));
  });
  frame.contentDocument.close();
}, "Event listeners are to be removed from shadow trees as well");

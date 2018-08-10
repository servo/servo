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

test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.contentWindow.addEventListener("x", t.unreached_func("window event listener not removed"));
  frame.contentDocument.open();
  frame.contentWindow.dispatchEvent(new Event("x"));
  frame.contentDocument.close();
}, "Custom event listeners are to be removed from Window");

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

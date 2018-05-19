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
}, "Event listeners are to be removed");

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

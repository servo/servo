// https://dom.spec.whatwg.org/#concept-event-dispatch

const host = document.createElement("div"),
      child = host.appendChild(document.createElement("p")),
      shadow = host.attachShadow({ mode: "closed" }),
      slot = shadow.appendChild(document.createElement("slot"));

test(() => {
  for (target of [shadow, slot]) {
    for (relatedTarget of [new XMLHttpRequest(), self, host]) {
      const event = new FocusEvent("demo", { relatedTarget: relatedTarget });
      target.dispatchEvent(event);
      assert_equals(event.target, null);
      assert_equals(event.relatedTarget, null);
    }
  }
}, "Reset if target pointed to a shadow tree");

test(() => {
  for (relatedTarget of [shadow, slot]) {
    for (target of [new XMLHttpRequest(), self, host]) {
      const event = new FocusEvent("demo", { relatedTarget: relatedTarget });
      target.dispatchEvent(event);
      assert_equals(event.target, null);
      assert_equals(event.relatedTarget, null);
    }
  }
}, "Reset if relatedTarget pointed to a shadow tree");

async_test(t => {
  const shadowChild = shadow.appendChild(document.createElement("div"));
  shadowChild.addEventListener("demo", t.step_func(() => document.body.appendChild(shadowChild)));
  const event = new FocusEvent("demo", { relatedTarget: new XMLHttpRequest() });
  shadowChild.dispatchEvent(event);
  assert_equals(shadowChild.parentNode, document.body);
  assert_equals(event.target, null);
  assert_equals(event.relatedTarget, null);
  shadowChild.remove();
  t.done();
}, "Reset if target pointed to a shadow tree pre-dispatch");

async_test(t => {
  const shadowChild = shadow.appendChild(document.createElement("div"));
  shadowChild.addEventListener("demo", t.step_func(() => document.body.appendChild(shadowChild)));
  const event = new FocusEvent("demo", { relatedTarget: shadowChild });
  document.body.dispatchEvent(event);
  assert_equals(shadowChild.parentNode, document.body);
  assert_equals(event.target, null);
  assert_equals(event.relatedTarget, null);
  shadowChild.remove();
  t.done();
}, "Reset if relatedTarget pointed to a shadow tree pre-dispatch");

async_test(t => {
  const event = new FocusEvent("heya", { relatedTarget: shadow, cancelable: true }),
        callback = t.unreached_func();
  host.addEventListener("heya", callback);
  t.add_cleanup(() => host.removeEventListener("heya", callback));
  event.preventDefault();
  assert_true(event.defaultPrevented);
  assert_false(host.dispatchEvent(event));
  assert_equals(event.target, null);
  assert_equals(event.relatedTarget, null);
  // Check that the dispatch flag is cleared
  event.initEvent("x");
  assert_equals(event.type, "x");
  t.done();
}, "Reset targets on early return");

async_test(t => {
  const input = document.body.appendChild(document.createElement("input")),
        event = new MouseEvent("click", { relatedTarget: shadow });
  let seen = false;
  t.add_cleanup(() => input.remove());
  input.type = "checkbox";
  input.oninput = t.step_func(() => {
    assert_equals(event.target, null);
    assert_equals(event.relatedTarget, null);
    assert_equals(event.composedPath().length, 0);
    seen = true;
  });
  assert_true(input.dispatchEvent(event));
  assert_true(seen);
  t.done();
}, "Reset targets before activation behavior");

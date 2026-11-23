// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/resources/testdriver-actions.js

function createSwitch(t, style = "") {
  assert_true("switch" in HTMLInputElement.prototype, "switch is supported");
  const input = document.body.appendChild(document.createElement("input"));
  t.add_cleanup(() => input.remove());
  input.type = "checkbox";
  input.switch = true;
  input.style.cssText = `margin: 40px; ${style}`;
  return input;
}

function recordEvents(input) {
  const events = [];
  for (const type of ["click", "input", "change"]) {
    input.addEventListener(type, () => events.push(type));
  }
  return events;
}

// Drags the pointer along the inline axis of input, through each of the given positions. A
// position is a fraction of the input's inline size, with 0 at its inline-start edge and 1 at its
// inline-end edge.
async function drag(input, positions) {
  const style = getComputedStyle(input);
  const rect = input.getBoundingClientRect();
  const vertical = !style.writingMode.startsWith("horizontal");
  const rtl = style.direction == "rtl";
  const inlineSize = vertical ? rect.height : rect.width;
  const point = position => {
    const offset = Math.round((rtl ? 0.5 - position : position - 0.5) * inlineSize);
    return vertical ? [0, offset] : [offset, 0];
  };

  const actions = new test_driver.Actions().pointerMove(...point(positions[0]), { origin: input });
  actions.pointerDown();
  for (const position of positions.slice(1)) {
    actions.pointerMove(...point(position), { origin: input });
  }
  await actions.pointerUp().send();
}

promise_test(async t => {
  const input = createSwitch(t);
  const events = recordEvents(input);

  await drag(input, [0.15, 0.5, 0.85, 1.5]);
  assert_true(input.checked);
  assert_array_equals(events, ["click", "input", "change"]);
}, "Dragging the thumb to the other end and releasing outside the switch toggles it");

promise_test(async t => {
  const input = createSwitch(t);
  input.checked = true;
  const events = recordEvents(input);

  await drag(input, [0.85, 0.5, 0.15, -0.5]);
  assert_false(input.checked);
  assert_array_equals(events, ["click", "input", "change"]);
}, "Dragging the thumb of a switch that is on to the other end and releasing outside the switch toggles it");

promise_test(async t => {
  const input = createSwitch(t);
  const events = recordEvents(input);

  await drag(input, [0.15, 0.5, 0.85, 0.5, 0.15]);
  assert_false(input.checked);
  assert_array_equals(events, []);
}, "Dragging the thumb to the other end and back does not fire a click event");

promise_test(async t => {
  const input = createSwitch(t);
  const events = recordEvents(input);

  await drag(input, [0.15, 0.3, 0.15]);
  assert_true(input.checked);
  assert_array_equals(events, ["click", "input", "change"]);
}, "Moving the thumb without reaching the other end is a click");

promise_test(async t => {
  const input = createSwitch(t);
  const events = recordEvents(input);

  await drag(input, [0.15, -0.5]);
  assert_false(input.checked);
  assert_array_equals(events, []);
}, "Releasing outside the switch without reaching the other end does not fire a click event");

promise_test(async t => {
  const input = createSwitch(t);
  const events = recordEvents(input);
  input.addEventListener("pointerdown", () => {
    input.addEventListener("pointermove", () => { input.checked = true; }, { once: true });
  }, { once: true });

  await drag(input, [0.15, 0.5, 0.85, 1.5]);
  assert_true(input.checked);
  assert_array_equals(events, []);
}, "Setting checked during a drag stops the drag");

promise_test(async t => {
  const input = createSwitch(t);
  const events = recordEvents(input);
  input.addEventListener("pointerdown", () => {
    input.addEventListener("pointermove", () => { input.disabled = true; }, { once: true });
  }, { once: true });

  await drag(input, [0.15, 0.5, 0.85, 1.5]);
  assert_false(input.checked);
  assert_array_equals(events, []);
}, "Disabling the switch during a drag stops the drag");

promise_test(async t => {
  const input = createSwitch(t);
  const events = recordEvents(input);
  input.addEventListener("pointerdown", () => {
    input.addEventListener("pointermove", () => { input.removeAttribute("switch"); }, { once: true });
  }, { once: true });

  await drag(input, [0.15, 0.5, 0.85, 1.5]);
  assert_false(input.checked);
  assert_array_equals(events, []);
}, "Removing the switch attribute during a drag stops the drag");

promise_test(async t => {
  const input = createSwitch(t);
  const events = recordEvents(input);
  input.addEventListener("pointerdown", () => {
    input.addEventListener("pointermove", () => { input.type = "radio"; }, { once: true });
  }, { once: true });

  await drag(input, [0.15, 0.5, 0.85, 1.5]);
  assert_false(input.checked);
  assert_array_equals(events, []);
}, "Changing the type during a drag stops the drag");

promise_test(async t => {
  const input = createSwitch(t);
  input.dir = "rtl";
  const events = recordEvents(input);

  await drag(input, [0.15, 0.5, 0.85, 1.5]);
  assert_true(input.checked);
  assert_array_equals(events, ["click", "input", "change"]);
}, "Dragging the thumb of a right-to-left switch from right to left toggles it");

promise_test(async t => {
  const input = createSwitch(t, "writing-mode: vertical-lr");
  const events = recordEvents(input);

  await drag(input, [0.15, 0.5, 0.85, 1.5]);
  assert_true(input.checked);
  assert_array_equals(events, ["click", "input", "change"]);
}, "Dragging the thumb of a vertical switch from top to bottom toggles it");

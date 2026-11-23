// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/resources/testdriver-actions.js

function createSwitch(t) {
  assert_true("switch" in HTMLInputElement.prototype, "switch is supported");
  const input = document.body.appendChild(document.createElement("input"));
  t.add_cleanup(() => input.remove());
  input.type = "checkbox";
  input.switch = true;
  return input;
}

function recordEvents(input) {
  const events = [];
  for (const type of ["click", "input", "change"]) {
    input.addEventListener(type, () => events.push(type));
  }
  return events;
}

test(t => {
  const input = createSwitch(t);
  const events = recordEvents(input);

  input.click();
  assert_true(input.checked);
  assert_array_equals(events, ["click", "input", "change"]);

  input.click();
  assert_false(input.checked);
  assert_array_equals(events, ["click", "input", "change", "click", "input", "change"]);
}, "click() toggles a switch and fires click, input, and change");

promise_test(async t => {
  const input = createSwitch(t);
  const events = recordEvents(input);

  await test_driver.click(input);
  assert_true(input.checked);
  assert_array_equals(events, ["click", "input", "change"]);
}, "Clicking a switch toggles it and fires click, input, and change once");

promise_test(async t => {
  const input = createSwitch(t);
  input.disabled = true;
  input.style.margin = "20px";
  const events = recordEvents(input);
  const offset = Math.round(input.getBoundingClientRect().width * 0.35);

  await new test_driver.Actions()
    .pointerMove(-offset, 0, { origin: input })
    .pointerDown()
    .pointerMove(offset, 0, { origin: input })
    .pointerUp()
    .send();
  assert_false(input.checked);
  assert_array_equals(events, []);
}, "Dragging across a disabled switch does not toggle it");

promise_test(async t => {
  const input = createSwitch(t);
  await test_driver.click(input);
  assert_true(input.checked);

  input.removeAttribute("switch");
  const events = recordEvents(input);
  await test_driver.click(input);
  assert_false(input.checked);
  assert_array_equals(events, ["click", "input", "change"]);
}, "Clicking a switch whose switch attribute was removed toggles it like a checkbox");

test(t => {
  const input = createSwitch(t);
  input.addEventListener("click", e => e.preventDefault());
  const events = recordEvents(input);

  input.click();
  assert_false(input.checked);
  assert_array_equals(events, ["click"]);
}, "Canceling the click event reverts a switch and fires neither input nor change");

test(t => {
  const input = createSwitch(t);
  const label = document.body.appendChild(document.createElement("label"));
  t.add_cleanup(() => label.remove());
  label.append(input);

  label.click();
  assert_true(input.checked);
}, "Clicking the label of a switch toggles it");

test(t => {
  const input = createSwitch(t);
  input.indeterminate = true;
  assert_true(input.indeterminate);

  input.click();
  assert_true(input.checked);
  assert_false(input.indeterminate);
}, "A switch keeps its indeterminateness until a click resets it");

test(t => {
  const input = createSwitch(t);
  const form = document.body.appendChild(document.createElement("form"));
  t.add_cleanup(() => form.remove());
  form.append(input);
  input.name = "s";
  assert_equals(new FormData(form).get("s"), null);

  input.checked = true;
  assert_equals(new FormData(form).get("s"), "on");

  input.value = "yes";
  assert_equals(new FormData(form).get("s"), "yes");
}, "A switch contributes to form data like a checkbox");

test(t => {
  const input = createSwitch(t);
  input.required = true;
  assert_true(input.validity.valueMissing);

  input.checked = true;
  assert_false(input.validity.valueMissing);
}, "A required switch suffers from being missing while it is off");

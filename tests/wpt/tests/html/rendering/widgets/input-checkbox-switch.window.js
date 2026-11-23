test(t => {
  const input = document.body.appendChild(document.createElement("input"));
  t.add_cleanup(() => input.remove());
  input.type = "checkbox";
  input.switch = true;
  assert_true(input.hasAttribute("switch"), "switch is supported");
  assert_equals(getComputedStyle(input).appearance, "auto");
}, "Default appearance value");

test(t => {
  const input = document.body.appendChild(document.createElement("input"));
  t.add_cleanup(() => input.remove());
  input.type = "checkbox";
  input.switch = true;
  assert_true(input.hasAttribute("switch"), "switch is supported");
  input.style.display = "none"
  assert_equals(getComputedStyle(input).display, "none");
}, "Default appearance value: display:none");

test(t => {
  const input = document.body.appendChild(document.createElement("input"));
  t.add_cleanup(() => input.remove());
  input.type = "checkbox";
  input.switch = true;
  assert_true(input.hasAttribute("switch"), "switch is supported");
  input.style.appearance = "none";
  assert_equals(getComputedStyle(input).appearance, "none");
}, "appearance:none should work");

test(t => {
  const input = document.body.appendChild(document.createElement("input"));
  t.add_cleanup(() => input.remove());
  input.type = "checkbox";
  input.switch = true;
  assert_true(input.hasAttribute("switch"), "switch is supported");
  input.style.appearance = "none";
  assert_equals(getComputedStyle(input).display, "inline-block");
}, "appearance:none should work: display is inline-block");

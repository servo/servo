test(t => {
  const input = document.body.appendChild(document.createElement("input"));
  t.add_cleanup(() => input.remove());
  input.type = "checkbox";
  input.switch = true;
  assert_equals(getComputedStyle(input).appearance, "auto");
}, "Default appearance value");

test(t => {
  const input = document.body.appendChild(document.createElement("input"));
  t.add_cleanup(() => input.remove());
  input.type = "checkbox";
  input.switch = true;
  input.style.appearance = "none";
  assert_equals(getComputedStyle(input).appearance, "none");
}, "appearance:none should work");

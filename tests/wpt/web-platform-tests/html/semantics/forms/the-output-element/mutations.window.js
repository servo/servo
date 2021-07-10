function assert_equal_values(output, value, message) {
  assert_equals(output.value, value, `.value ${message}`);
  assert_equals(output.defaultValue, value, `.defaultValue ${message}`);
}
function assert_values(output, value, defaultValue, message) {
  assert_equals(output.value, value, `.value ${message}`);
  assert_equals(output.defaultValue, defaultValue, `.defaultValue ${message}`);
}

test(() => {
  const output = document.createElement("output"),
        child = output.appendChild(document.createElement("span"));
  assert_equal_values(output, "", "start");
  child.textContent = "x";
  assert_equal_values(output, "x", "after setting textContent");
  output.value = "some";
  assert_values(output, "some", "x", "after setting value");
  child.textContent = "y";
  assert_values(output, "y", "x", "after setting textContent again");
}, "Descendant mutations and output.value and .defaultValue");

test(() => {
  const form = document.createElement("form"),
        output = form.appendChild(document.createElement("output"));
  output.textContent = "value";
  assert_equal_values(output, "value", "after setting textContent");
  output.value = "heya";
  assert_values(output, "heya", "value", "after setting value");
  form.reset();
  assert_equal_values(output, "value", "after form.reset()");

  output.innerHTML = "<div>something</div>";
  assert_equal_values(output, "something", "after setting innerHTML");
  form.reset();
  assert_equal_values(output, "something", "after form.reset() again");
}, "output and output.form.reset()");

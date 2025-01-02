test(() => {
  const input = document.createElement("input");
  input.type = "checkbox";

  assert_false(input.matches(":checked:indeterminate"));
  assert_false(input.matches(":checked"));
  assert_false(input.matches(":indeterminate"));

  input.checked = true;
  input.indeterminate = true;

  assert_true(input.matches(":checked:indeterminate"));
  assert_true(input.matches(":checked"));
  assert_true(input.matches(":indeterminate"));

  input.indeterminate = false;

  assert_false(input.matches(":checked:indeterminate"));
  assert_true(input.matches(":checked"));
  assert_false(input.matches(":indeterminate"));

  input.checked = false;

  assert_false(input.matches(":checked:indeterminate"));
  assert_false(input.matches(":checked"));
  assert_false(input.matches(":indeterminate"));
}, "An element can be :checked and :indeterminate at the same time");

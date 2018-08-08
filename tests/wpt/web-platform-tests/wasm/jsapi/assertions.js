function assert_function_name(fn, name, description) {
  const propdesc = Object.getOwnPropertyDescriptor(fn, "name");
  assert_equals(typeof propdesc, "object", `${description} should have name property`);
  assert_false(propdesc.writable, "writable", `${description} name should not be writable`);
  assert_false(propdesc.enumerable, "enumerable", `${description} name should not be enumerable`);
  assert_true(propdesc.configurable, "configurable", `${description} name should be configurable`);
  assert_equals(propdesc.value, name, `${description} name should be ${name}`);
}

function assert_function_length(fn, length, description) {
  const propdesc = Object.getOwnPropertyDescriptor(fn, "length");
  assert_equals(typeof propdesc, "object", `${description} should have length property`);
  assert_false(propdesc.writable, "writable", `${description} length should not be writable`);
  assert_false(propdesc.enumerable, "enumerable", `${description} length should not be enumerable`);
  assert_true(propdesc.configurable, "configurable", `${description} length should be configurable`);
  assert_equals(propdesc.value, length, `${description} length should be ${length}`);
}

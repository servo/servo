// META: global=window,dedicatedworker,jsshell
// META: script=/wasm/jsapi/assertions.js

function assert_type(argument) {
  const exception = new WebAssembly.Exception(argument);
  const exceptiontype = exception.type();

  assert_array_equals(exceptiontype.parameters, argument.parameters);
}

test(() => {
  assert_type({ "parameters": [] });
}, "[]");

test(() => {
  assert_type({ "parameters": ["i32", "i64"] });
}, "[i32 i64]");

test(() => {
  assert_type({ "parameters": ["i32", "i64", "f32", "f64"] });
}, "[i32 i64 f32 f64]");

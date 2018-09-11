// META: global=jsshell

test(() => {
  const argument = { "value": "i32" };
  const global = new WebAssembly.Global(argument);
  assert_class_string(global, "WebAssembly.Global");
}, "Object.prototype.toString on an Global");

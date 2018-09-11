// META: global=jsshell

test(() => {
  const argument = { "initial": 0 };
  const memory = new WebAssembly.Memory(argument);
  assert_class_string(memory, "WebAssembly.Memory");
}, "Object.prototype.toString on an Memory");

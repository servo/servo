// META: global=window,dedicatedworker,jsshell

test(() => {
  const thisValues = [
    undefined,
    null,
    true,
    "",
    Symbol(),
    1,
    {},
    WebAssembly.Global,
    WebAssembly.Global.prototype,
  ];

  const desc = Object.getOwnPropertyDescriptor(WebAssembly.Global.prototype, "value");
  assert_equals(typeof desc, "object");

  const getter = desc.get;
  assert_equals(typeof getter, "function");

  const setter = desc.set;
  assert_equals(typeof setter, "function");

  for (const thisValue of thisValues) {
    assert_throws_js(TypeError, () => getter.call(thisValue), `getter with this=${format_value(thisValue)}`);
    assert_throws_js(TypeError, () => setter.call(thisValue, 1), `setter with this=${format_value(thisValue)}`);
  }
}, "Branding");

for (const type of ["i32", "f32", "f64"]) {
  const immutableOptions = [
    [{}, "missing"],
    [{ "mutable": undefined }, "undefined"],
    [{ "mutable": null }, "null"],
    [{ "mutable": false }, "false"],
    [{ "mutable": "" }, "empty string"],
    [{ "mutable": 0 }, "zero"],
  ];
  for (const [opts, name] of immutableOptions) {
    test(() => {
      opts.value = type;
      const global = new WebAssembly.Global(opts);
      assert_equals(global.value, 0, "initial value");
      assert_equals(global.valueOf(), 0, "initial valueOf");

      assert_throws_js(TypeError, () => global.value = 1);

      assert_equals(global.value, 0, "post-set value");
      assert_equals(global.valueOf(), 0, "post-set valueOf");
    }, `Immutable ${type} (${name})`);

    test(t => {
      opts.value = type;
      const global = new WebAssembly.Global(opts);
      assert_equals(global.value, 0, "initial value");
      assert_equals(global.valueOf(), 0, "initial valueOf");

      const value = {
        valueOf: t.unreached_func("should not call valueOf"),
        toString: t.unreached_func("should not call toString"),
      };
      assert_throws_js(TypeError, () => global.value = value);

      assert_equals(global.value, 0, "post-set value");
      assert_equals(global.valueOf(), 0, "post-set valueOf");
    }, `Immutable ${type} with ToNumber side-effects (${name})`);
  }

  const mutableOptions = [
    [{ "mutable": true }, "true"],
    [{ "mutable": 1 }, "one"],
    [{ "mutable": "x" }, "string"],
    [Object.create({ "mutable": true }), "true on prototype"],
  ];
  for (const [opts, name] of mutableOptions) {
    test(() => {
      opts.value = type;
      const global = new WebAssembly.Global(opts);
      assert_equals(global.value, 0, "initial value");
      assert_equals(global.valueOf(), 0, "initial valueOf");

      global.value = 1;

      assert_equals(global.value, 1, "post-set value");
      assert_equals(global.valueOf(), 1, "post-set valueOf");
    }, `Mutable ${type} (${name})`);
  }
}

test(() => {
  const argument = { "value": "i64", "mutable": true };
  const global = new WebAssembly.Global(argument);
  assert_throws_js(TypeError, () => global.value);
  assert_throws_js(TypeError, () => global.value = 0);
  assert_throws_js(TypeError, () => global.valueOf());
}, "i64 with default");

test(t => {
  const argument = { "value": "i64", "mutable": true };
  const global = new WebAssembly.Global(argument);
  const value = {
    valueOf: t.unreached_func("should not call valueOf"),
    toString: t.unreached_func("should not call toString"),
  };
  assert_throws_js(TypeError, () => global.value = value);
}, "i64 with ToNumber side-effects");

test(() => {
  const argument = { "value": "i32", "mutable": true };
  const global = new WebAssembly.Global(argument);
  const desc = Object.getOwnPropertyDescriptor(WebAssembly.Global.prototype, "value");
  assert_equals(typeof desc, "object");

  const setter = desc.set;
  assert_equals(typeof setter, "function");

  assert_throws_js(TypeError, () => setter.call(global));
}, "Calling setter without argument");

test(() => {
  const argument = { "value": "i32", "mutable": true };
  const global = new WebAssembly.Global(argument);
  const desc = Object.getOwnPropertyDescriptor(WebAssembly.Global.prototype, "value");
  assert_equals(typeof desc, "object");

  const getter = desc.get;
  assert_equals(typeof getter, "function");

  const setter = desc.set;
  assert_equals(typeof setter, "function");

  assert_equals(getter.call(global, {}), 0);
  assert_equals(setter.call(global, 1, {}), undefined);
  assert_equals(global.value, 1);
}, "Stray argument");

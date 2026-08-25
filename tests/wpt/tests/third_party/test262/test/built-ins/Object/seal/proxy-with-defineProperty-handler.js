// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.seal
description: >
  [[DefineOwnProperty]] is called with partial descriptor with only [[Configurable]] field present.
info: |
  SetIntegrityLevel ( O, level )

  [...]
  5. Let keys be ? O.[[OwnPropertyKeys]]().
  6. If level is sealed, then
    a. For each element k of keys, do
      i. Perform ? DefinePropertyOrThrow(O, k, PropertyDescriptor { [[Configurable]]: false }).
features: [Symbol, Proxy, Reflect]
---*/

var sym = Symbol();
var seenDescriptors = {};
var proxy = new Proxy({
  [sym]: 1,
  get foo() {},
  set foo(_v) {},
}, {
  defineProperty: function(target, key, descriptor) {
    seenDescriptors[key] = descriptor;
    return Reflect.defineProperty(target, key, descriptor);
  },
});

Object.seal(proxy);

assert.sameValue(seenDescriptors[sym].value, undefined, "value");
assert.sameValue(seenDescriptors[sym].writable, undefined, "writable");
assert.sameValue(seenDescriptors[sym].enumerable, undefined, "enumerable");
assert.sameValue(seenDescriptors[sym].configurable, false, "configurable");

assert.sameValue(seenDescriptors.foo.get, undefined, "get");
assert.sameValue(seenDescriptors.foo.set, undefined, "set");
assert.sameValue(seenDescriptors.foo.enumerable, undefined, "enumerable");
assert.sameValue(seenDescriptors.foo.configurable, false, "configurable");

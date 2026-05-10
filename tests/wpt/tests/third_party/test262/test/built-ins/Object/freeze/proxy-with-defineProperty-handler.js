// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.freeze
description: >
  [[DefineOwnProperty]] is called with partial descriptor with only [[Configurable]] and
  [[Writable]] (for data properties only) fields present.
info: |
  SetIntegrityLevel ( O, level )

  [...]
  5. Let keys be ? O.[[OwnPropertyKeys]]().
  [...]
  7. Else,
    a. Assert: level is frozen.
    b. For each element k of keys, do
      i. Let currentDesc be ? O.[[GetOwnProperty]](k).
      ii. If currentDesc is not undefined, then
        1. If IsAccessorDescriptor(currentDesc) is true, then
          a. Let desc be the PropertyDescriptor { [[Configurable]]: false }.
        2. Else,
          a. Let desc be the PropertyDescriptor { [[Configurable]]: false, [[Writable]]: false }.
        3. Perform ? DefinePropertyOrThrow(O, k, desc).
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

Object.freeze(proxy);

assert.sameValue(seenDescriptors[sym].value, undefined, "value");
assert.sameValue(seenDescriptors[sym].writable, false, "writable");
assert.sameValue(seenDescriptors[sym].enumerable, undefined, "enumerable");
assert.sameValue(seenDescriptors[sym].configurable, false, "configurable");

assert.sameValue(seenDescriptors.foo.get, undefined, "get");
assert.sameValue(seenDescriptors.foo.set, undefined, "set");
assert.sameValue(seenDescriptors.foo.enumerable, undefined, "enumerable");
assert.sameValue(seenDescriptors.foo.configurable, false, "configurable");

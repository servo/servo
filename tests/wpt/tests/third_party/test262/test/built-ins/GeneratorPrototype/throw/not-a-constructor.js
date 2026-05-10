// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-ecmascript-standard-built-in-objects
description: >
  Object.getPrototypeOf((function* g() {}).prototype).throw does not implement [[Construct]], is not new-able
info: |
  ECMAScript Function Objects

  Built-in function objects that are not identified as constructors do not
  implement the [[Construct]] internal method unless otherwise specified in
  the description of a particular function.

  sec-evaluatenew

  ...
  7. If IsConstructor(constructor) is false, throw a TypeError exception.
  ...
includes: [isConstructor.js]
features: [Reflect.construct, generators, arrow-function]
---*/

assert.sameValue(
  isConstructor(Object.getPrototypeOf((function* g() {}).prototype).throw),
  false,
  'isConstructor(Object.getPrototypeOf((function* g() {}).prototype).throw) must return false'
);

assert.throws(TypeError, () => {
  function* g() {} let expected = new Test262Error(); let iterator = g();try {new iterator.throw(expected);} catch (e) {if (e !== expected) {throw e;}}
});


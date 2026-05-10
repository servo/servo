// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-symbol-constructor
description: >
  Symbol is a constructor and has [[Construct]] internal method.
info: |
  The Symbol constructor:
  [...]
  * may be used as the value of an extends clause of a class definition
  but a super call to it will cause an exception.
  [...]
  * has a [[Prototype]] internal slot whose value is %Function.prototype%.
includes: [isConstructor.js]
features: [Symbol, Reflect.construct]
---*/

assert(isConstructor(Symbol));
assert.sameValue(Object.getPrototypeOf(Symbol), Function.prototype);

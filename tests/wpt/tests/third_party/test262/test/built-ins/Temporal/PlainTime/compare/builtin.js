// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.compare
description: Tests that Temporal.PlainTime.compare meets the requirements for built-in objects
info: |
    Built-in functions that are not constructors do not have a "prototype" property unless
    otherwise specified in the description of a particular function.

    Unless specified otherwise, a built-in object that is callable as a function is a built-in
    function object with the characteristics described in 10.3. Unless specified otherwise, the
    [[Extensible]] internal slot of a built-in object initially has the value true.

    Unless otherwise specified every built-in function and every built-in constructor has the
    Function prototype object [...] as the value of its [[Prototype]] internal slot.
features: [Temporal]
---*/

assert.sameValue(Object.isExtensible(Temporal.PlainTime.compare),
  true, "Built-in objects must be extensible.");

assert.sameValue(Object.prototype.toString.call(Temporal.PlainTime.compare),
  "[object Function]", "Object.prototype.toString");

assert.sameValue(Object.getPrototypeOf(Temporal.PlainTime.compare),
  Function.prototype, "prototype");

assert.sameValue(Temporal.PlainTime.compare.hasOwnProperty("prototype"),
  false, "prototype property");

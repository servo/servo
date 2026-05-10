// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tolocalestring
description: >
    Tests that Temporal.PlainDate.prototype.toLocaleString
    meets the requirements for built-in objects defined by the
    introduction of chapter 17 of the ECMAScript Language Specification.
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

assert.sameValue(Object.isExtensible(Temporal.PlainDate.prototype.toLocaleString),
  true, "Built-in objects must be extensible.");

assert.sameValue(Object.prototype.toString.call(Temporal.PlainDate.prototype.toLocaleString),
  "[object Function]", "Object.prototype.toString");

assert.sameValue(Object.getPrototypeOf(Temporal.PlainDate.prototype.toLocaleString),
  Function.prototype, "prototype");

assert.sameValue(Temporal.PlainDate.prototype.toLocaleString.hasOwnProperty("prototype"),
  false, "prototype property");

// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: The prototype of the Intl.RelativeTimeFormat constructor is %FunctionPrototype%.
info: |
    Unless specified otherwise in this document, the objects, functions, and constructors described in this standard are subject to the generic requirements and restrictions specified for standard built-in ECMAScript objects in the ECMAScript 2019 Language Specification, 10th edition, clause 17, or successor.
    Unless otherwise specified every built-in function object has the %FunctionPrototype% object as the initial value of its [[Prototype]] internal slot.
features: [Intl.RelativeTimeFormat]
---*/

assert.sameValue(
  Object.getPrototypeOf(Intl.RelativeTimeFormat),
  Function.prototype,
  "Object.getPrototypeOf(Intl.RelativeTimeFormat) equals the value of Function.prototype"
);

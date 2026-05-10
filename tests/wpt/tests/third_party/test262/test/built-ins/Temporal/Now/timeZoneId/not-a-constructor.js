// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.now.timezoneid
description: Temporal.Now.timeZoneId does not implement [[Construct]]
info: |
  ECMAScript Function Objects

  Built-in function objects that are not identified as constructors do not
  implement the [[Construct]] internal method unless otherwise specified in
  the description of a particular function.
includes: [isConstructor.js]
features: [Reflect.construct, Temporal, arrow-function]
---*/

assert.sameValue(isConstructor(Temporal.Now.timeZoneId), false, 'isConstructor(Temporal.Now.timeZoneId) must return false');

assert.throws(TypeError, () => {
  new Temporal.Now.timeZoneId();
}, 'new Temporal.Now.timeZoneId() throws a TypeError exception');

// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.now.plaindatetimeiso
description: Temporal.Now.plainDateTimeISO does not implement [[Construct]]
includes: [isConstructor.js]
features: [Reflect.construct, Temporal, arrow-function]
---*/

assert.sameValue(isConstructor(Temporal.Now.plainDateTimeISO), false, 'isConstructor(Temporal.Now.plainDateTimeISO) must return false');

assert.throws(TypeError, () => {
  new Temporal.Now.plainDateTimeISO();
}, 'new Temporal.Now.plainDateTimeISO() throws a TypeError exception');

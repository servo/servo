// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.now.zoneddatetimeiso
description: Temporal.Now.zonedDateTimeISO does not implement [[Construct]]
includes: [isConstructor.js]
features: [Reflect.construct, Temporal, arrow-function]
---*/

assert.sameValue(isConstructor(Temporal.Now.zonedDateTimeISO), false, 'isConstructor(Temporal.Now.zonedDateTimeISO) must return false');

assert.throws(TypeError, () => {
  new Temporal.Now.zonedDateTimeISO();
}, 'new Temporal.Now.zonedDateTimeISO() throws a TypeError exception');

// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.now.instant
description: Temporal.Now.instant does not implement [[Construct]]
includes: [isConstructor.js]
features: [Reflect.construct, Temporal, arrow-function]
---*/

assert.sameValue(isConstructor(Temporal.Now.instant), false, 'isConstructor(Temporal.Now.instant) must return false');

assert.throws(TypeError, () => {
  new Temporal.Now.instant();
}, 'new Temporal.Now.instant() throws a TypeError exception');

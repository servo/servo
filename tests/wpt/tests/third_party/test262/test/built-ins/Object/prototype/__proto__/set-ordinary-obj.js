// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.__proto__
es6id: B.2.2.1
description: Setting valid value on an ordinary object
info: |
    [...]
    4. Let status be ? O.[[SetPrototypeOf]](proto).
    5. If status is false, throw a TypeError exception.
    6. Return undefined.
features: [__proto__]
---*/

var set = Object.getOwnPropertyDescriptor(Object.prototype, '__proto__').set;
var proto = {};
var subject = {};

assert.sameValue(set.call(subject, proto), undefined);
assert.sameValue(Object.getPrototypeOf(subject), proto);

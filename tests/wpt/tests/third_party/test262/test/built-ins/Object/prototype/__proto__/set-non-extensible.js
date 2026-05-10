// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.__proto__
es6id: B.2.2.1
description: Called on an non-extensible object
info: |
    [...]
    4. Let status be ? O.[[SetPrototypeOf]](proto).
    5. If status is false, throw a TypeError exception.

    9.1.2.1 OrdinarySetPrototypeOf

    [...]
    2. Let extensible be the value of the [[Extensible]] internal slot of O.
    3. Let current be the value of the [[Prototype]] internal slot of O.
    4. If SameValue(V, current) is true, return true.
    5. If extensible is false, return false.
features: [__proto__]
---*/

var proto = {};
var subject = Object.create(proto);

Object.preventExtensions(subject);

subject.__proto__ = proto;

assert.throws(TypeError, function() {
  subject.__proto__ = {};
});

assert.sameValue(Object.getPrototypeOf(subject), proto);

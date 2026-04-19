// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.__proto__
es6id: B.2.2.1
description: Normal completion from ordinary object's [[GetPrototypeOf]]
info: |
    1. Let O be ? ToObject(this value).
    2. Return ? O.[[GetPrototypeOf]]().
features: [__proto__]
---*/

var get = Object.getOwnPropertyDescriptor(Object.prototype, '__proto__').get;
var proto = {};
var withCustomProto = Object.create(proto);
var withNullProto = Object.create(null);

assert.sameValue(get.call({}), Object.prototype, 'Ordinary object');
assert.sameValue(get.call(withCustomProto), proto, 'custom prototype object');
assert.sameValue(get.call(withNullProto), null, 'null prototype');

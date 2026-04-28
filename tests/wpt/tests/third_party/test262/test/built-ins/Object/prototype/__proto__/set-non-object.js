// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.__proto__
es6id: B.2.2.1
description: Called on a value that is object-coercible but not an Object
info: |
    1. Let O be ? RequireObjectCoercible(this value).
    2. If Type(proto) is neither Object nor Null, return undefined.
    3. If Type(O) is not Object, return undefined.
features: [Symbol, __proto__]
---*/

var set = Object.getOwnPropertyDescriptor(Object.prototype, '__proto__').set;
var subject = {};

assert.sameValue(set.call(true), undefined, 'boolean');
assert.sameValue(
  Object.getPrototypeOf(subject), Object.prototype, 'following boolean'
);

assert.sameValue(set.call(1), undefined, 'number');
assert.sameValue(
  Object.getPrototypeOf(subject), Object.prototype, 'following number'
);

assert.sameValue(set.call('string'), undefined, 'string');
assert.sameValue(
  Object.getPrototypeOf(subject), Object.prototype, 'following string'
);

assert.sameValue(set.call(Symbol('')), undefined, 'symbol');
assert.sameValue(
  Object.getPrototypeOf(subject), Object.prototype, 'following symbol'
);

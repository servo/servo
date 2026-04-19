// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.__proto__
es6id: B.2.2.1
description: Abrupt completion from [[GetPrototypeOf]]
info: |
    1. Let O be ? ToObject(this value).
    2. Return ? O.[[GetPrototypeOf]]().
features: [Proxy, __proto__]
---*/

var get = Object.getOwnPropertyDescriptor(Object.prototype, '__proto__').get;
var thrower = function() {
  throw new Test262Error();
};

var subject = new Proxy({}, { getPrototypeOf: thrower });

assert.throws(Test262Error, function() {
  get.call(subject);
});

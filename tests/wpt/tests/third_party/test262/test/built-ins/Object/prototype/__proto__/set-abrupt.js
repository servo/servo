// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.__proto__
es6id: B.2.2.1
description: Abrupt completion from [[SetPrototypeOf]]
info: |
    [...]
    4. Let status be ? O.[[SetPrototypeOf]](proto).
features: [Proxy, __proto__]
---*/

var thrower = function() {
  throw new Test262Error();
};
var subject = new Proxy({}, { setPrototypeOf: thrower });

assert.throws(Test262Error, function() {
  subject.__proto__ = {};
});

assert.sameValue(Object.getPrototypeOf(subject), Object.prototype);

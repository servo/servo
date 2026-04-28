// Copyright (C) 2017 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
author: Jordan Harband
description: >
  Promise.prototype.finally called with a `this` value that defines a `then`
  method which returns an abrupt completion.
esid: sec-promise.prototype.finally
features: [Promise.prototype.finally]
---*/

var thrower = new Promise(function() {});
thrower.then = function() {
  throw new Test262Error();
};

assert.throws(Test262Error, function() {
  Promise.prototype.finally.call(thrower);
});

assert.throws(Test262Error, function() {
  thrower.finally();
});

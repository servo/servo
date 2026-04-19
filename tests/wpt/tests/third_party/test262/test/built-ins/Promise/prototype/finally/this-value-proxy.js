// Copyright (C) 2018 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
author: Jordan Harband
description: >
  Promise.prototype.finally called with a non-branded Promise does not throw
esid: sec-promise.prototype.finally
features: [Promise.prototype.finally]
---*/

var called = false;
var p = new Proxy(Promise.resolve(), {});
var oldThen = Promise.prototype.then;
Promise.prototype.then = () => {
  called = true;
};
Promise.prototype.finally.call(p);
assert.sameValue(called, true);
Promise.prototype.then = oldThen;

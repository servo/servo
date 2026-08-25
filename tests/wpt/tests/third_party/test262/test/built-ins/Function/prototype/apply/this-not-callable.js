// Copyright 2009 the Sputnik authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function.prototype.apply
description: >
  Throws a TypeError exception if this value is not callable
info: |
  Function.prototype.apply ( thisArg, argArray )

  1. Let func be the this value.
  2. If IsCallable(func) is false, throw a TypeError exception.
---*/

assert.throws(TypeError, function() {
  Function.prototype.apply.call(undefined, {}, []);
});

assert.throws(TypeError, function() {
  Function.prototype.apply.call(null, {}, []);
});

assert.throws(TypeError, function() {
  Function.prototype.apply.call({}, {}, []);
});

assert.throws(TypeError, function() {
  Function.prototype.apply.call(/re/, {}, []);
});

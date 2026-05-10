// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.4.5
description: >
  Resolve function is called after Promise constructor returns.
info: |
  Promise.resolve ( x )

  ...
  4. Let promiseCapability be NewPromiseCapability(C).
  5. ReturnIfAbrupt(promiseCapability).
  6. Let resolveResult be Call(promiseCapability.[[Resolve]], undefined, «x»).
  7. ReturnIfAbrupt(resolveResult).
  ...
---*/

var expectedThisValue = (function() {
  return this;
}());
var callCount = 0;
var object = {};
var thisValue, args;

Promise.resolve.call(function(executor) {
  function resolve(v) {
    callCount += 1;
    thisValue = this;
    args = arguments;
  }
  executor(resolve, Test262Error.thrower);
  assert.sameValue(callCount, 0, "callCount before returning from constructor");
}, object);

assert.sameValue(callCount, 1, "callCount after call to resolve()");
assert.sameValue(typeof args, "object");
assert.sameValue(args.length, 1);
assert.sameValue(args[0], object);
assert.sameValue(thisValue, expectedThisValue);

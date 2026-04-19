// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.4.5
description: >
  Throws a TypeError if capabilities executor already called with non-undefined values.
info: |
  Promise.resolve ( x )

  ...
  4. Let promiseCapability be NewPromiseCapability(C).
  5. ReturnIfAbrupt(promiseCapability).
  ...

  25.4.1.5.1 GetCapabilitiesExecutor Functions
    ...
    3. If promiseCapability.[[Resolve]] is not undefined, throw a TypeError exception.
    4. If promiseCapability.[[Reject]] is not undefined, throw a TypeError exception.
    5. Set promiseCapability.[[Resolve]] to resolve.
    6. Set promiseCapability.[[Reject]] to reject.
    ...
---*/

var checkPoint = "";
Promise.resolve.call(function(executor) {
  checkPoint += "a";
  executor();
  checkPoint += "b";
  executor(function() {}, function() {});
  checkPoint += "c";
}, {});
assert.sameValue(checkPoint, "abc", "executor initially called with no arguments");

var checkPoint = "";
Promise.resolve.call(function(executor) {
  checkPoint += "a";
  executor(undefined, undefined);
  checkPoint += "b";
  executor(function() {}, function() {});
  checkPoint += "c";
}, {});
assert.sameValue(checkPoint, "abc", "executor initially called with (undefined, undefined)");

var checkPoint = "";
assert.throws(TypeError, function() {
  Promise.resolve.call(function(executor) {
    checkPoint += "a";
    executor(undefined, function() {});
    checkPoint += "b";
    executor(function() {}, function() {});
    checkPoint += "c";
  }, {});
}, "executor initially called with (undefined, function)");
assert.sameValue(checkPoint, "ab", "executor initially called with (undefined, function)");

var checkPoint = "";
assert.throws(TypeError, function() {
  Promise.resolve.call(function(executor) {
    checkPoint += "a";
    executor(function() {}, undefined);
    checkPoint += "b";
    executor(function() {}, function() {});
    checkPoint += "c";
  }, {});
}, "executor initially called with (function, undefined)");
assert.sameValue(checkPoint, "ab", "executor initially called with (function, undefined)");

var checkPoint = "";
assert.throws(TypeError, function() {
  Promise.resolve.call(function(executor) {
    checkPoint += "a";
    executor("invalid value", 123);
    checkPoint += "b";
    executor(function() {}, function() {});
    checkPoint += "c";
  }, {});
}, "executor initially called with (String, Number)");
assert.sameValue(checkPoint, "ab", "executor initially called with (String, Number)");

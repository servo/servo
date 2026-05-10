// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.5.3
description: >
  Throws a TypeError if either resolve or reject capability is not callable.
info: |
  Promise.prototype.then ( onFulfilled , onRejected )

  ...
  5. Let promiseCapability be NewPromiseCapability(C).
  6. ReturnIfAbrupt(promiseCapability).
  ...

  25.4.1.5 NewPromiseCapability ( C )
    ...
    4. Let executor be a new built-in function object as defined in GetCapabilitiesExecutor Functions (25.4.1.5.1).
    5. Set the [[Capability]] internal slot of executor to promiseCapability.
    6. Let promise be Construct(C, «executor»).
    7. ReturnIfAbrupt(promise).
    8. If IsCallable(promiseCapability.[[Resolve]]) is false, throw a TypeError exception.
    9. If IsCallable(promiseCapability.[[Reject]]) is false, throw a TypeError exception.
    ...
features: [class]
---*/

var constructorFunction;

var promise = new class extends Promise {
  constructor(executor) {
    if (constructorFunction) {
      constructorFunction(executor);
      return {};
    }
    return super(executor);
  }
}(function() {});

var checkPoint = "";
assert.throws(TypeError, function() {
  constructorFunction = function(executor) {
    checkPoint += "a";
  };
  promise.then();
}, "executor not called at all");
assert.sameValue(checkPoint, "a", "executor not called at all");

var checkPoint = "";
assert.throws(TypeError, function() {
  constructorFunction = function(executor) {
    checkPoint += "a";
    executor();
    checkPoint += "b";
  };
  promise.then();
}, "executor called with no arguments");
assert.sameValue(checkPoint, "ab", "executor called with no arguments");

var checkPoint = "";
assert.throws(TypeError, function() {
  constructorFunction = function(executor) {
    checkPoint += "a";
    executor(undefined, undefined);
    checkPoint += "b";
  };
  promise.then();
}, "executor called with (undefined, undefined)");
assert.sameValue(checkPoint, "ab", "executor called with (undefined, undefined)");

var checkPoint = "";
assert.throws(TypeError, function() {
  constructorFunction = function(executor) {
    checkPoint += "a";
    executor(undefined, function() {});
    checkPoint += "b";
  };
  promise.then();
}, "executor called with (undefined, function)");
assert.sameValue(checkPoint, "ab", "executor called with (undefined, function)");

var checkPoint = "";
assert.throws(TypeError, function() {
  constructorFunction = function(executor) {
    checkPoint += "a";
    executor(function() {}, undefined);
    checkPoint += "b";
  };
  promise.then();
}, "executor called with (function, undefined)");
assert.sameValue(checkPoint, "ab", "executor called with (function, undefined)");

var checkPoint = "";
assert.throws(TypeError, function() {
  constructorFunction = function(executor) {
    checkPoint += "a";
    executor(123, "invalid value");
    checkPoint += "b";
  };
  promise.then();
}, "executor called with (Number, String)");
assert.sameValue(checkPoint, "ab", "executor called with (Number, String)");

// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.4.1.1
description: >
  Cannot tamper remainingElementsCount when Promise.all resolve element function is called twice in a row.
info: |
  Runtime Semantics: PerformPromiseAll( iteratorRecord, constructor, resultCapability)

  ...
  4. Let remainingElementsCount be a new Record { [[value]]: 1 }.
  ...
  6.d ...
    ii. Set remainingElementsCount.[[value]] to remainingElementsCount.[[value]] − 1.
    iii. If remainingElementsCount.[[value]] is 0,
      1. Let valuesArray be CreateArrayFromList(values).
      2. Let resolveResult be Call(resultCapability.[[Resolve]], undefined, «valuesArray»).
      3. ReturnIfAbrupt(resolveResult).
  ...

  25.4.4.1.2 Promise.all Resolve Element Functions
    1. Let alreadyCalled be the value of F's [[AlreadyCalled]] internal slot.
    2. If alreadyCalled.[[value]] is true, return undefined.
    3. Set alreadyCalled.[[value]] to true.
    ...
---*/

var callCount = 0;

function Constructor(executor) {
  function resolve(values) {
    callCount += 1;
    assert(Array.isArray(values), "values is array");
    assert.sameValue(values.length, 3, "values length");
    assert.sameValue(values[0], "p1-fulfill", "values[0]");
    assert.sameValue(values[1], "p2-fulfill", "values[1]");
    assert.sameValue(values[2], "p3-fulfill", "values[2]");
  }
  executor(resolve, Test262Error.thrower);
}
Constructor.resolve = function(v) {
  return v;
};

var p1OnFulfilled;

var p1 = {
  then: function(onFulfilled, onRejected) {
    p1OnFulfilled = onFulfilled;
  }
};
var p2 = {
  then: function(onFulfilled, onRejected) {
    onFulfilled("p2-fulfill");
    onFulfilled("p2-fulfill-unexpected");
  }
};
var p3 = {
  then: function(onFulfilled, onRejected) {
    onFulfilled("p3-fulfill");
  }
};

assert.sameValue(callCount, 0, "callCount before call to all()");

Promise.all.call(Constructor, [p1, p2, p3]);

assert.sameValue(callCount, 0, "callCount after call to all()");

p1OnFulfilled("p1-fulfill");

assert.sameValue(callCount, 1, "callCount after resolving p1");

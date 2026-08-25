// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.sumprecise
description: Math.sumPrecise throws and closes the iterator if any element is not a Number
features: [Math.sumPrecise]
---*/

assert.throws(TypeError, function () {
  Math.sumPrecise([{}]);
});

assert.throws(TypeError, function () {
  Math.sumPrecise([0n]);
});


var coercions = 0;
var objectWithValueOf = {
  valueOf: function() {
    ++coercions;
    throw new Test262Error("valueOf should not be called");
  },
  toString: function() {
    ++coercions;
    throw new Test262Error("toString should not be called");
  }
};

assert.throws(TypeError, function () {
  Math.sumPrecise([objectWithValueOf]);
});
assert.sameValue(coercions, 0);

assert.throws(TypeError, function () {
  Math.sumPrecise([objectWithValueOf, NaN]);
});
assert.sameValue(coercions, 0);

assert.throws(TypeError, function () {
  Math.sumPrecise([NaN, objectWithValueOf]);
});
assert.sameValue(coercions, 0);

assert.throws(TypeError, function () {
  Math.sumPrecise([-Infinity, Infinity, objectWithValueOf]);
});
assert.sameValue(coercions, 0);

var nextCalls = 0;
var returnCalls = 0;
var iterator = {
  next: function () {
    ++nextCalls;
    return { done: false, value: objectWithValueOf };
  },
  return: function () {
    ++returnCalls;
    return {};
  }
};
var iterable = {
  [Symbol.iterator]: function () {
    return iterator;
  }
};

assert.throws(TypeError, function () {
  Math.sumPrecise(iterable);
});
assert.sameValue(coercions, 0);
assert.sameValue(nextCalls, 1);
assert.sameValue(returnCalls, 1);

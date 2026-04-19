// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map-iterable
description: >
  Closes the iterator after `not Object` error.
info: |
  Map ( [ iterable ] )

  ...
  9. Repeat
    ...
    d. Let nextItem be IteratorValue(next).
    e. ReturnIfAbrupt(nextItem).
    f. If Type(nextItem) is not Object,
      i. Let error be Completion{[[type]]: throw, [[value]]: a newly created
      TypeError object, [[target]]:empty}.
      ii. Return IteratorClose(iter, error).
features:
  - Symbol
  - Symbol.iterator
---*/

var count = 0;
var nextItem;
var iterable = {};
iterable[Symbol.iterator] = function() {
  return {
    next: function() {
      return {
        value: nextItem,
        done: false
      };
    },
    return: function() {
      count += 1;
    }
  };
};

nextItem = 1;
assert.throws(TypeError, function() {
  new Map(iterable);
});
assert.sameValue(count, 1);

nextItem = true;
assert.throws(TypeError, function() {
  new Map(iterable);
});
assert.sameValue(count, 2);

nextItem = '';
assert.throws(TypeError, function() {
  new Map(iterable);
});
assert.sameValue(count, 3);

nextItem = null;
assert.throws(TypeError, function() {
  new Map(iterable);
});
assert.sameValue(count, 4);

nextItem = undefined;
assert.throws(TypeError, function() {
  new Map(iterable);
});
assert.sameValue(count, 5);

nextItem = Symbol('a');
assert.throws(TypeError, function() {
  new Map(iterable);
});
assert.sameValue(count, 6);

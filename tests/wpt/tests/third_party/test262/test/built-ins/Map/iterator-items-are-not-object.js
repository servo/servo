// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map-iterable
description: >
  Throws a TypeError if iterable items are not Objects.
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
features: [Symbol]
---*/

assert.throws(TypeError, function() {
  new Map([1]);
});

assert.throws(TypeError, function() {
  new Map(['']);
});

assert.throws(TypeError, function() {
  new Map([true]);
});

assert.throws(TypeError, function() {
  new Map([null]);
});

assert.throws(TypeError, function() {
  new Map([Symbol('a')]);
});

assert.throws(TypeError, function() {
  new Map([undefined]);
});

assert.throws(TypeError, function() {
  new Map([
    ['a', 1],
    2
  ]);
});

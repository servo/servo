// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset-iterable
description: >
  If the iterable argument is empty, return new Weakset object.
info: |
  23.4.1.1 WeakSet ( [ iterable ] )

  ...
  9. Repeat
    a. Let next be IteratorStep(iter).
    b. ReturnIfAbrupt(next).
    c. If next is false, return set.
  ...
---*/

var counter = 0;
var add = WeakSet.prototype.add;
WeakSet.prototype.add = function(value) {
  counter++;
  return add.call(this, value);
};
var set = new WeakSet([]);

assert.sameValue(Object.getPrototypeOf(set), WeakSet.prototype);
assert(set instanceof WeakSet);
assert.sameValue(
  counter, 0,
  'empty iterable does not call WeakSet.prototype.add'
);

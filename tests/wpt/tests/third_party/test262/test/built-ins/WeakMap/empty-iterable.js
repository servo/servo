// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap-iterable
description: >
  If the iterable argument is empty, return new WeakMap object.
info: |
  23.3.1.1 WeakMap ( [ iterable ] )

  ...
  9. Repeat
    a. Let next be IteratorStep(iter).
    b. ReturnIfAbrupt(next).
    c. If next is false, return map.
  ...
---*/

var counter = 0;
var set = WeakMap.prototype.set;
WeakMap.prototype.set = function(value) {
  counter++;
  return set.call(this, value);
};
var map = new WeakMap([]);

assert.sameValue(Object.getPrototypeOf(map), WeakMap.prototype);
assert(map instanceof WeakMap);
assert.sameValue(
  counter, 0,
  'empty iterable does not call WeakMap.prototype.set'
);

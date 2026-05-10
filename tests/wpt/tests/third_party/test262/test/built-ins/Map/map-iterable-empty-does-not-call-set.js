// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map-iterable
description: >
  A Map constructed with an empty iterable argument does not call set.
info: |
  Map ( [ iterable ] )

  When the Map function is called with optional argument the following steps are
  taken:

  ...
  8. If iter is undefined, return map.
  9. Repeat
    a. Let next be IteratorStep(iter).
    b. ReturnIfAbrupt(next).
    c. If next is false, return map.
---*/

var set = Map.prototype.set;
var counter = 0;

Map.prototype.set = function(value) {
  counter++;
  set.call(this, value);
};

new Map([]);

assert.sameValue(counter, 0, '`Map.prototype.set` was not called.');

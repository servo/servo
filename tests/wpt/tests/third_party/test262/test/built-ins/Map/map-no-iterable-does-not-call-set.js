// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map-iterable
description: >
  A Map constructed without a iterable argument does not call set.
info: |
  Map ( [ iterable ] )

  When the Map function is called with optional argument the following steps are
  taken:

  ...
  5. If iterable is not present, let iterable be undefined.
  6. If iterable is either undefined or null, let iter be undefined.
  7. Else,
    a. Let adder be Get(map, "set").
    b. ReturnIfAbrupt(adder).
    c. If IsCallable(adder) is false, throw a TypeError exception.
    d. Let iter be GetIterator(iterable).
    e. ReturnIfAbrupt(iter).
  8. If iter is undefined, return map.
---*/

var set = Map.prototype.set;
var counter = 0;

Map.prototype.set = function(value) {
  counter++;
  set.call(this, value);
};

new Map();

assert.sameValue(counter, 0, '`Map.prototype.set` was not called.');

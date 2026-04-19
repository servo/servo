// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map-iterable
description: >
  Throws a TypeError if `set` is not callable on Map constructor with a
  iterable argument.
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
---*/

Map.prototype.set = null;

assert.throws(TypeError, function() {
  new Map([
    [1, 1],
    [2, 2]
  ]);
});

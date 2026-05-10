// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map-iterable
description: >
  Creating a new Map object without arguments doesn't throw if `set` is not
  callable
info: |
  Map ( [ iterable ] )

  When the Set function is called with optional argument iterable the following steps are taken:

  ...
  5. If iterable is not present, let iterable be undefined.
  6. If iterable is either undefined or null, let iter be undefined.
  7. Else,
    a. Let adder be Get(map, "set").
    b. ReturnIfAbrupt(adder).
    c. If IsCallable(adder) is false, throw a TypeError exception.
    ...
  8. If iter is undefined, return map.
  ...
---*/

Map.prototype.set = null;

var m = new Map();

assert.sameValue(m.size, 0, 'The value of `m.size` is `0`');

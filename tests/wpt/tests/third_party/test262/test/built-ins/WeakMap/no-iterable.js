// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap-iterable
description: >
  If the iterable argument is undefined, return new WeakMap object.
info: |
  WeakMap ( [ iterable ] )

  ...
  5. If iterable is not present, let iterable be undefined.
  6. If iterable is either undefined or null, let iter be undefined.
  ...
  8. If iter is undefined, return map.
  ...
---*/

var a = new WeakMap();
var b = new WeakMap(undefined);
var c = new WeakMap(null);

assert.sameValue(Object.getPrototypeOf(a), WeakMap.prototype);
assert.sameValue(Object.getPrototypeOf(b), WeakMap.prototype);
assert.sameValue(Object.getPrototypeOf(c), WeakMap.prototype);

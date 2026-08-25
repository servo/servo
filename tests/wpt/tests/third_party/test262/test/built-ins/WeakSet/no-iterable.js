// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset-iterable
description: >
  If the iterable argument is undefined, return new Weakset object.
info: |
  23.4.1.1 WeakSet ( [ iterable ] )

  ...
  5. If iterable is not present, let iterable be undefined.
  6. If iterable is either undefined or null, let iter be undefined.
  ...
  8. If iter is undefined, return set.
  ...
---*/

var a = new WeakSet();
var b = new WeakSet(undefined);
var c = new WeakSet(null);

assert.sameValue(Object.getPrototypeOf(a), WeakSet.prototype);
assert.sameValue(Object.getPrototypeOf(b), WeakSet.prototype);
assert.sameValue(Object.getPrototypeOf(c), WeakSet.prototype);

// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map-iterable
description: >
  The new Map object's prototype is Map.prototype
info: |
  Map ( [ iterable ] )

  When the Map function is called with optional argument the following steps
  are taken:

  ...
  2. Let map be OrdinaryCreateFromConstructor(NewTarget, "%MapPrototype%",
  «‍[[MapData]]» ).
  ...

---*/

var m1 = new Map();

assert.sameValue(
  Object.getPrototypeOf(m1),
  Map.prototype,
  "`Object.getPrototypeOf(m1)` returns `Map.prototype`"
);

var m2 = new Map([
  [1, 1],
  [2, 2]
]);

assert.sameValue(
  Object.getPrototypeOf(m2),
  Map.prototype,
  "`Object.getPrototypeOf(m2)` returns `Map.prototype`"
);

// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map-iterable
description: >
  Returns the new Map object with the new empty list if the iterable argument is
  undefined.
info: |
  Map ( [ iterable ] )

  ...
  2. Let map be OrdinaryCreateFromConstructor(NewTarget, "%MapPrototype%",
  «‍[[MapData]]» ).
  ...
  4. Map map’s [[MapData]] internal slot to a new empty List.
  5. If iterable is not present, let iterable be undefined.
  6. If iterable is either undefined or null, let iter be undefined.
  ...
  8. If iter is undefined, return map.
---*/

var m1 = new Map();
var m2 = new Map(undefined);
var m3 = new Map(null);

assert.sameValue(m1.size, 0, 'The value of `new Map().size` is `0`');
assert.sameValue(m2.size, 0, 'The value of `new Map(undefined).size` is `0`');
assert.sameValue(m3.size, 0, 'The value of `new Map(null).size` is `0`');

assert(m1 instanceof Map);
assert(m2 instanceof Map);
assert(m3 instanceof Map);

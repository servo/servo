// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4
description: Uint32Array traversal using for..of
info: |
    Uint32Array instances should be able to be traversed using a `for..of`
    loop.
features: [TypedArray]
---*/

var iterationCount = 0;
var array = new Uint32Array([3, 2, 4, 1]);

var first = 3;
var second = 2;
var third = 4;
var fourth = 1;

for (var x of array) {
  assert.sameValue(x, first);

  first = second;
  second = third;
  third = fourth;
  fourth = null;

  iterationCount += 1;
}

assert.sameValue(iterationCount, 4);

// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.add
description: >
    Set.prototype.add ( value )

    ...
    7. Append value as the last element of entries.
    ...
---*/

var s = new Set();
var expects = [1, 2, 3];

s.add(1).add(2).add(3);

s.forEach(function(value) {
  assert.sameValue(value, expects.shift());
});

assert.sameValue(expects.length, 0, "The value of `expects.length` is `0`");

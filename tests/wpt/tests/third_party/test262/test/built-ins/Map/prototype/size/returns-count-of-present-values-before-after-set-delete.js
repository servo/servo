// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-map.prototype.size
description: >
  Returns count of present values before and after using `set` and `delete`.
info: |
  get Map.prototype.size

  5. Let count be 0.
  6. For each Record {[[key]], [[value]]} p that is an element of entries
    a. If p.[[key]] is not empty, set count to count+1.
---*/

var map = new Map();

assert.sameValue(map.size, 0, 'The value of `map.size` is `0`');

map.set(1, 1);
assert.sameValue(
  map.size, 1,
  'The value of `map.size` is `1`, after executing `map.set(1, 1)`'
);

map.delete(1);
assert.sameValue(
  map.size, 0,
  'The value of `map.size` is `0`, after executing `map.delete(1)`'
);

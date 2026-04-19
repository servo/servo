// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-map.prototype.size
description: >
  Returns count of present values inserted with set.
info: |
  get Map.prototype.size

  5. Let count be 0.
  6. For each Record {[[key]], [[value]]} p that is an element of entries
    a. If p.[[key]] is not empty, set count to count+1.
features: [Symbol]
---*/

var map = new Map();

map.set(0, undefined);
map.set(undefined, undefined);
map.set(false, undefined);
map.set(NaN, undefined);
map.set(null, undefined);
map.set('', undefined);
map.set(Symbol(), undefined);

assert.sameValue(map.size, 7, 'The value of `map.size` is `7`');

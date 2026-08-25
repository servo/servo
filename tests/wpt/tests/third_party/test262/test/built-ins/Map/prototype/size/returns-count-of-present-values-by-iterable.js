// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-map.prototype.size
description: >
  Returns count of present values inserted via iterable argument.
info: |
  get Map.prototype.size

  5. Let count be 0.
  6. For each Record {[[key]], [[value]]} p that is an element of entries
    a. If p.[[key]] is not empty, set count to count+1.
features: [Symbol]
---*/

var map = new Map([
  [0, undefined],
  [undefined, undefined],
  [false, undefined],
  [NaN, undefined],
  [null, undefined],
  ['', undefined],
  [Symbol(), undefined],
]);

assert.sameValue(map.size, 7, 'The value of `map.size` is `7`');

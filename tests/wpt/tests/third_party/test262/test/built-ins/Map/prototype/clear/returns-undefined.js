// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.clear
description: >
  Returns undefined.
info: |
  Map.prototype.clear ( )

  ...
  4. Let entries be the List that is the value of M’s [[MapData]] internal slot.
  5. Repeat for each Record {[[key]], [[value]]} p that is an element of
  entries,
    a. Set p.[[key]] to empty.
    b. Set p.[[value]] to empty.
  6. Return undefined.
---*/

var m1 = new Map([
  ['foo', 'bar'],
  [1, 1]
]);

assert.sameValue(m1.clear(), undefined, 'clears a map and returns undefined');
assert.sameValue(m1.clear(), undefined, 'returns undefined on an empty map');

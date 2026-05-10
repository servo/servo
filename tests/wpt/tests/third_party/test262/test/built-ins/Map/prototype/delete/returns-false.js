// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.delete
description: >
  Returns false when it does not delete an entry.
info: |
  Map.prototype.delete ( key )

  4. Let entries be the List that is the value of M’s [[MapData]] internal slot.
  5. Repeat for each Record {[[key]], [[value]]} p that is an element of entries,
    a. If p.[[key]] is not empty and SameValueZero(p.[[key]], key) is true, then
      ...
      iii. Return true.
  6. Return false.
---*/

var m = new Map([
  ['a', 1],
  ['b', 2]
]);

assert.sameValue(m.delete('not-in-the-map'), false);

m.delete('a');
assert.sameValue(m.delete('a'), false);

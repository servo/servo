// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.clear
description: >
  The existing [[MapData]] List is preserved.
info: |
  The existing [[MapData]] List is preserved because there may be existing
  MapIterator objects that are suspended midway through iterating over that
  List.

  Map.prototype.clear ( )

  ...
  4. Let entries be the List that is the value of M’s [[MapData]] internal slot.
  5. Repeat for each Record {[[key]], [[value]]} p that is an element of
  entries,
    a. Set p.[[key]] to empty.
    b. Set p.[[value]] to empty.
  6. Return undefined.
---*/

var m = new Map([
  [1, 1],
  [2, 2],
  [3, 3]
]);
var e = m.entries();

e.next();
m.clear();

var n = e.next();
assert.sameValue(n.value, undefined);
assert.sameValue(n.done, true);

// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.set
description: >
  Replaces a value in the map.
info: |
  Map.prototype.set ( key , value )

  ...
  5. Repeat for each Record {[[key]], [[value]]} p that is an element of
  entries,
    a. If p.[[key]] is not empty and SameValueZero(p.[[key]], key) is true, then
      i. Set p.[[value]] to value.
      ii. Return M.
  ...
---*/

var m = new Map([['item', 1]]);

m.set('item', 42);
assert.sameValue(m.get('item'), 42);
assert.sameValue(m.size, 1);

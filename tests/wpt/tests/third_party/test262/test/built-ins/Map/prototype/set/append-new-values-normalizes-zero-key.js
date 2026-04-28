// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.set
description: >
  Appends new value in the map normalizing +0 and -0.
info: |
  Map.prototype.set ( key , value )

  ...
  6. If key is −0, let key be +0.
  7. Let p be the Record {[[key]]: key, [[value]]: value}.
  8. Append p as the last element of entries.
  9. Return M.
  ...
---*/

var map = new Map();
map.set(-0, 42);

assert.sameValue(map.get(0), 42);

map = new Map();
map.set(+0, 43);
assert.sameValue(map.get(0), 43);

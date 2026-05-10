// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.set
description: >
  Map.prototype.set returns the given `this` object.
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
var result = map.set(1, 1);

assert.sameValue(result, map);

result = map.set(1,1).set(2,2).set(3,3);

assert.sameValue(result, map, 'Map.prototype.set is chainable');

var map2 = new Map();
result = map2.set.call(map, 4, 4);

assert.sameValue(result, map);

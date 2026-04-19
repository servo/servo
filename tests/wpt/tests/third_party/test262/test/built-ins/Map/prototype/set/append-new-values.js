// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.set
description: >
  Append a new value as the last element of entries.
info: |
  Map.prototype.set ( key , value )

  ...
  6. If key is −0, let key be +0.
  7. Let p be the Record {[[key]]: key, [[value]]: value}.
  8. Append p as the last element of entries.
  9. Return M.
  ...
features: [Symbol]
---*/

var s = Symbol(2);
var map = new Map([[4, 4], ['foo3', 3], [s, 2]]);

map.set(null, 42);
map.set(1, 'valid');

assert.sameValue(map.size, 5);
assert.sameValue(map.get(1), 'valid');

var results = [];

map.forEach(function(value, key) {
  results.push({
    value: value,
    key: key
  });
});

var result = results.pop();
assert.sameValue(result.value, 'valid');
assert.sameValue(result.key, 1);

result = results.pop();
assert.sameValue(result.value, 42);
assert.sameValue(result.key, null);

result = results.pop();
assert.sameValue(result.value, 2);
assert.sameValue(result.key, s);

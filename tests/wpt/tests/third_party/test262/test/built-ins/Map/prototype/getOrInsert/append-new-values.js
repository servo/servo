// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Jonas Haukenes. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsert
description: |
  Append a new value as the last element of entries.
info: |
  Map.prototype.getOrInsert ( key , value )

  ...
  3. Set key to CanonicalizeKeyedCollectionKey(key).
  4. For each Record { [[Key]], [[Value]] } p of M.[[MapData]], do
    a. If p.[[Key]] is not empty and SameValue(p.[[Key]], key) is true, return p.[[Value]].
  5. Let p be the Record { [[Key]]: key, [[Value]]: value }.
  6. Append p to M.[[MapData]].
  ...
features: [Symbol, upsert]
---*/
var s = Symbol(2);
var map = new Map([[4, 4], ['foo3', 3], [s, 2]]);

map.getOrInsert(null, 42);
map.getOrInsert(1, 'valid');

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


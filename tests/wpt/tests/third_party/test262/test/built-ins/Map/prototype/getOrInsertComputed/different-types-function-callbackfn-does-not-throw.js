// Copyright (C) 2024 Mathias Ness. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsertcomputed
description: |
  Does not throw if `callbackfn` is callable.
info: |
  Map.prototype.getOrInsertComputed ( key , callbackfn )

   ...
  3. If IsCallable(callbackfn) is false, throw a TypeError exception.
  ...

features: [arrow-function, upsert]
---*/
var m = new Map();

assert.sameValue(m.getOrInsertComputed(1, function () { return 1; }), 1);
assert.sameValue(m.get(1), 1);

assert.sameValue(m.getOrInsertComputed(2, () => 2), 2);
assert.sameValue(m.get(2), 2);

function three() { return 3; }
assert.sameValue(m.getOrInsertComputed(3, three), 3);
assert.sameValue(m.get(3), 3);

assert.sameValue(m.getOrInsertComputed(4, new Function()), undefined);
assert.sameValue(m.get(4), undefined);

assert.sameValue(m.getOrInsertComputed(5, (function () { return 5; }).bind(m)), 5);
assert.sameValue(m.get(5), 5);

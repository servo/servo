// Copyright (C) 2025 Jonas Haukenes, Mathias Ness. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.getorinsertcomputed
description: |
  Does not throw if `callbackfn` is callable.
info: |
  WeakMap.prototype.getOrInsertComputed ( key , callbackfn )

  ...
  3. If IsCallable(callbackfn) is false, throw a TypeError exception.
  ...
features: [arrow-function, upsert, WeakMap]
---*/
var bar = {};
var baz = {};
var foo = {};
var foobar = {};
var foobarbaz = {};

var m = new WeakMap();

assert.sameValue(
    m.getOrInsertComputed(bar, function() {return 1;})
    , 1);
assert.sameValue(m.get(bar), 1);


assert.sameValue(
    m.getOrInsertComputed(baz, () => 2)
    , 2);
assert.sameValue(m.get(baz), 2);


function three() {return 3;}

assert.sameValue(
    m.getOrInsertComputed(foo, three)
    , 3);
assert.sameValue(m.get(foo), 3);


assert.sameValue(
    m.getOrInsertComputed(foobar, new Function())
    , undefined);
assert.sameValue(m.get(foobar), undefined);


assert.sameValue(
    m.getOrInsertComputed(foobarbaz, (function() {return 5;}).bind(m))
    , 5);
assert.sameValue(m.get(foobarbaz), 5);



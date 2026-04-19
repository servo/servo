// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Behavior when regexp[@@matchAll] is not callable
info: |
  String.prototype.matchAll ( regexp )
    [...]
    2. If regexp is neither undefined nor null, then
      a. Let matcher be ? GetMethod(regexp, @@matchAll).
features: [Symbol.matchAll, String.prototype.matchAll]
---*/

assert.sameValue(typeof String.prototype.matchAll, "function");

var regexp = /./;

regexp[Symbol.matchAll] = true;
assert.throws(TypeError, function() {
  ''.matchAll(regexp);
});

regexp[Symbol.matchAll] = 5;
assert.throws(TypeError, function() {
  ''.matchAll(regexp);
});

regexp[Symbol.matchAll] = '';
assert.throws(TypeError, function() {
  ''.matchAll(regexp);
});

regexp[Symbol.matchAll] = Symbol();
assert.throws(TypeError, function() {
  ''.matchAll(regexp);
});

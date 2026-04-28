// Copyright (C) 2021 Ron Buckton and Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.flags
description: Boolean coercion of the hasIndices property
info: |
  get RegExp.prototype.flags

  ...
  10. Let hasIndices be ToBoolean(? Get(R, "hasIndices")).
  ...
features: [Symbol, regexp-match-indices]
---*/

var get = Object.getOwnPropertyDescriptor(RegExp.prototype, "flags").get;

var r = {};

r.hasIndices = undefined;
assert.sameValue(get.call(r), "", "hasIndices: undefined");

r.hasIndices = null;
assert.sameValue(get.call(r), "", "hasIndices: null");

r.hasIndices = NaN;
assert.sameValue(get.call(r), "", "hasIndices: NaN");

r.hasIndices = "";
assert.sameValue(get.call(r), "", "hasIndices: the empty string");

r.hasIndices = "string";
assert.sameValue(get.call(r), "d", "hasIndices: string");

r.hasIndices = 86;
assert.sameValue(get.call(r), "d", "hasIndices: 86");

r.hasIndices = Symbol();
assert.sameValue(get.call(r), "d", "hasIndices: Symbol()");

r.hasIndices = [];
assert.sameValue(get.call(r), "d", "hasIndices: []");

r.hasIndices = {};
assert.sameValue(get.call(r), "d", "hasIndices: {}");

// Copyright (C) 2017 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.flags
description: Boolean coercion of the dotAll property
info: |
  get RegExp.prototype.flags

  ...
  10. Let dotAll be ToBoolean(? Get(R, "dotAll")).
  ...
features: [Symbol, regexp-dotall]
---*/

var get = Object.getOwnPropertyDescriptor(RegExp.prototype, "flags").get;

var r = {};

r.dotAll = undefined;
assert.sameValue(get.call(r), "", "dotAll: undefined");

r.dotAll = null;
assert.sameValue(get.call(r), "", "dotAll: null");

r.dotAll = NaN;
assert.sameValue(get.call(r), "", "dotAll: NaN");

r.dotAll = "";
assert.sameValue(get.call(r), "", "dotAll: the empty string");

r.dotAll = "string";
assert.sameValue(get.call(r), "s", "dotAll: string");

r.dotAll = 86;
assert.sameValue(get.call(r), "s", "dotAll: 86");

r.dotAll = Symbol();
assert.sameValue(get.call(r), "s", "dotAll: Symbol()");

r.dotAll = [];
assert.sameValue(get.call(r), "s", "dotAll: []");

r.dotAll = {};
assert.sameValue(get.call(r), "s", "dotAll: {}");

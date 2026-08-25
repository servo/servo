// Copyright (C) 2017 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.flags
description: Boolean coercion of the global property
info: |
  get RegExp.prototype.flags

  ...
  4. Let global be ToBoolean(? Get(R, "global")).
  ...
features: [Symbol]
---*/

var get = Object.getOwnPropertyDescriptor(RegExp.prototype, "flags").get;

var r = {};

r.global = undefined;
assert.sameValue(get.call(r), "", "global: undefined");

r.global = null;
assert.sameValue(get.call(r), "", "global: null");

r.global = NaN;
assert.sameValue(get.call(r), "", "global: NaN");

r.global = "";
assert.sameValue(get.call(r), "", "global: the empty string");

r.global = "string";
assert.sameValue(get.call(r), "g", "global: string");

r.global = 86;
assert.sameValue(get.call(r), "g", "global: 86");

r.global = Symbol();
assert.sameValue(get.call(r), "g", "global: Symbol()");

r.global = [];
assert.sameValue(get.call(r), "g", "global: []");

r.global = {};
assert.sameValue(get.call(r), "g", "global: {}");

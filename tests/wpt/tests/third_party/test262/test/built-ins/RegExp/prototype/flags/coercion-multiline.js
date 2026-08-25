// Copyright (C) 2017 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.flags
description: Boolean coercion of the multiline property
info: |
  get RegExp.prototype.flags

  ...
  8. Let multiline be ToBoolean(? Get(R, "multiline")).
  ...
features: [Symbol]
---*/

var get = Object.getOwnPropertyDescriptor(RegExp.prototype, "flags").get;

var r = {};

r.multiline = undefined;
assert.sameValue(get.call(r), "", "multiline: undefined");

r.multiline = null;
assert.sameValue(get.call(r), "", "multiline: null");

r.multiline = NaN;
assert.sameValue(get.call(r), "", "multiline: NaN");

r.multiline = "";
assert.sameValue(get.call(r), "", "multiline: the empty string");

r.multiline = "string";
assert.sameValue(get.call(r), "m", "multiline: string");

r.multiline = 86;
assert.sameValue(get.call(r), "m", "multiline: 86");

r.multiline = Symbol();
assert.sameValue(get.call(r), "m", "multiline: Symbol()");

r.multiline = [];
assert.sameValue(get.call(r), "m", "multiline: []");

r.multiline = {};
assert.sameValue(get.call(r), "m", "multiline: {}");

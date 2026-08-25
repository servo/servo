// Copyright (C) 2017 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.flags
description: Boolean coercion of the unicode property
info: |
  get RegExp.prototype.flags

  ...
  12. Let unicode be ToBoolean(? Get(R, "unicode")).
  ...
features: [Symbol]
---*/

var get = Object.getOwnPropertyDescriptor(RegExp.prototype, "flags").get;

var r = {};

r.unicode = undefined;
assert.sameValue(get.call(r), "", "unicode: undefined");

r.unicode = null;
assert.sameValue(get.call(r), "", "unicode: null");

r.unicode = NaN;
assert.sameValue(get.call(r), "", "unicode: NaN");

r.unicode = "";
assert.sameValue(get.call(r), "", "unicode: the empty string");

r.unicode = "string";
assert.sameValue(get.call(r), "u", "unicode: string");

r.unicode = 86;
assert.sameValue(get.call(r), "u", "unicode: 86");

r.unicode = Symbol();
assert.sameValue(get.call(r), "u", "unicode: Symbol()");

r.unicode = [];
assert.sameValue(get.call(r), "u", "unicode: []");

r.unicode = {};
assert.sameValue(get.call(r), "u", "unicode: {}");

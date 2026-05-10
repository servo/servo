// Copyright (C) 2017 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.flags
description: Boolean coercion of the ignoreCase property
info: |
  get RegExp.prototype.flags

  ...
  6. Let ignoreCase be ToBoolean(? Get(R, "ignoreCase")).
  ...
features: [Symbol]
---*/

var get = Object.getOwnPropertyDescriptor(RegExp.prototype, "flags").get;

var r = {};

r.ignoreCase = undefined;
assert.sameValue(get.call(r), "", "ignoreCase: undefined");

r.ignoreCase = null;
assert.sameValue(get.call(r), "", "ignoreCase: null");

r.ignoreCase = NaN;
assert.sameValue(get.call(r), "", "ignoreCase: NaN");

r.ignoreCase = "";
assert.sameValue(get.call(r), "", "ignoreCase: the empty string");

r.ignoreCase = "string";
assert.sameValue(get.call(r), "i", "ignoreCase: string");

r.ignoreCase = 86;
assert.sameValue(get.call(r), "i", "ignoreCase: 86");

r.ignoreCase = Symbol();
assert.sameValue(get.call(r), "i", "ignoreCase: Symbol()");

r.ignoreCase = [];
assert.sameValue(get.call(r), "i", "ignoreCase: []");

r.ignoreCase = {};
assert.sameValue(get.call(r), "i", "ignoreCase: {}");

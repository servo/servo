// Copyright (C) 2017 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.flags
description: Boolean coercion of the sticky property
info: |
  get RegExp.prototype.flags

  ...
  14. Let sticky be ToBoolean(? Get(R, "sticky")).
  ...
features: [Symbol]
---*/

var get = Object.getOwnPropertyDescriptor(RegExp.prototype, "flags").get;

var r = {};

r.sticky = undefined;
assert.sameValue(get.call(r), "", "sticky: undefined");

r.sticky = null;
assert.sameValue(get.call(r), "", "sticky: null");

r.sticky = NaN;
assert.sameValue(get.call(r), "", "sticky: NaN");

r.sticky = "";
assert.sameValue(get.call(r), "", "sticky: the empty string");

r.sticky = "string";
assert.sameValue(get.call(r), "y", "sticky: string");

r.sticky = 86;
assert.sameValue(get.call(r), "y", "sticky: 86");

r.sticky = Symbol();
assert.sameValue(get.call(r), "y", "sticky: Symbol()");

r.sticky = [];
assert.sameValue(get.call(r), "y", "sticky: []");

r.sticky = {};
assert.sameValue(get.call(r), "y", "sticky: {}");

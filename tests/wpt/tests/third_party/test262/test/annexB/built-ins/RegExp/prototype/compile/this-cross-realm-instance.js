// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype.compile
description: RegExp.prototype.compile throws a TypeError for cross-realm calls
features: [legacy-regexp,cross-realm]
---*/

const other = $262.createRealm().global;

const regexp = new RegExp("");
const otherRealm_regexp = new other.RegExp("");

assert.throws(
  TypeError,
  function () {
    RegExp.prototype.compile.call(otherRealm_regexp);
  });

assert.throws(
  other.TypeError,
  function () {
    other.RegExp.prototype.compile.call(regexp);
  },
  "`other.RegExp.prototype.compile.call(regexp)` throws TypeError"
);

assert.sameValue(
  otherRealm_regexp.compile(),
  otherRealm_regexp,
  "`otherRealm_regexp.compile()` is SameValue with `otherRealm_regexp`"
);

// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.groupby
description: Object.groupBy called with non-callable throws TypeError
info: |
  Object.groupBy ( items, callbackfn )

  ...
  GroupBy ( items, callbackfn, coercion )

  2. If IsCallable(callbackfn) is false, throw a TypeError exception.
  ...
features: [array-grouping]
---*/


assert.throws(TypeError, function() {
  Object.groupBy([], null)
}, "null callback throws TypeError");

assert.throws(TypeError, function() {
  Object.groupBy([], undefined)
}, "undefined callback throws TypeError");

assert.throws(TypeError, function() {
  Object.groupBy([], {})
}, "object callback throws TypeError");

// Copyright 2024 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.escape
description: escape called with a RegExp object from another realm
features: [RegExp.escape, cross-realm]
---*/

const str = "oi+hello";
const other = $262.createRealm().global;

assert.sameValue(typeof other.RegExp.escape, "function", "other.RegExp.escape is a function");

const res = other.RegExp.escape.call(RegExp, str);

assert.sameValue(res, RegExp.escape(str), "cross-realm escape works correctly");

// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Property descriptor for RegExp.lastParen
info: |
  RegExp.lastParen is an accessor property with attributes:
  {
    [[Enumerable]]: false,
    [[Configurable]]: true,
    [[Set]]: undefined,
  }

  get RegExp.lastParen

  1. Return ? GetLegacyRegExpStaticProperty(%RegExp%, this value, [[RegExpLastParen]]).
includes: [propertyHelper.js]
features: [legacy-regexp]
---*/

var desc = Object.getOwnPropertyDescriptor(RegExp, "lastParen");

assert.sameValue(desc.set, undefined, "`set` property");
assert.sameValue(typeof desc.get, "function", "`get` property");

verifyProperty(RegExp, "lastParen", {
  enumerable: false,
  configurable: true
});

desc = Object.getOwnPropertyDescriptor(RegExp, "$+");

assert.sameValue(desc.set, undefined, "`set` property");
assert.sameValue(typeof desc.get, "function", "`get` property");

verifyProperty(RegExp, "$+", {
  enumerable: false,
  configurable: true
});

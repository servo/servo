// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Property descriptor for RegExp.lastMatch
info: |
  RegExp.lastMatch is an accessor property with attributes:
  {
    [[Enumerable]]: false,
    [[Configurable]]: true,
    [[Set]]: undefined,
  }

  get RegExp.lastMatch

  1. Return ? GetLegacyRegExpStaticProperty(%RegExp%, this value, [[RegExpLastMatch]]).
includes: [propertyHelper.js]
features: [legacy-regexp]
---*/

var desc = Object.getOwnPropertyDescriptor(RegExp, "lastMatch");

assert.sameValue(desc.set, undefined, "`set` property");
assert.sameValue(typeof desc.get, "function", "`get` property");

verifyProperty(RegExp, "lastMatch", {
  enumerable: false,
  configurable: true
});

desc = Object.getOwnPropertyDescriptor(RegExp, "$&");

assert.sameValue(desc.set, undefined, "`set` property");
assert.sameValue(typeof desc.get, "function", "`get` property");

verifyProperty(RegExp, "$&", {
  enumerable: false,
  configurable: true
});

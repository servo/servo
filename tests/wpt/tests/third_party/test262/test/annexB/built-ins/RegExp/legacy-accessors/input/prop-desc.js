// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Property descriptor for RegExp.input
info: |
  RegExp.input is an accessor property with attributes:
  {
    [[Enumerable]]: false,
    [[Configurable]]: true,
  }

  get RegExp.input

  1. Return ? GetLegacyRegExpStaticProperty(%RegExp%, this value, [[RegExpInput]]).

  set RegExp.input = val

  1. Return ? SetLegacyRegExpStaticProperty(%RegExp%, this value, [[RegExpInput]], val).
includes: [propertyHelper.js]
features: [legacy-regexp]
---*/

var desc = Object.getOwnPropertyDescriptor(RegExp, "input");

assert.sameValue(typeof desc.get, "function", "`get` property");
assert.sameValue(typeof desc.set, "function", "`set` property");

verifyProperty(RegExp, "input", {
  enumerable: false,
  configurable: true
});

desc = Object.getOwnPropertyDescriptor(RegExp, "$_");

assert.sameValue(typeof desc.get, "function", "`get` property");
assert.sameValue(typeof desc.set, "function", "`set` property");

verifyProperty(RegExp, "$_", {
  enumerable: false,
  configurable: true
});

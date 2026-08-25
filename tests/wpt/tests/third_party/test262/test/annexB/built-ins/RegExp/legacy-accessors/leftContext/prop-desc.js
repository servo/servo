// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Property descriptor for RegExp.leftContext
info: |
  RegExp.leftContext is an accessor property with attributes:
  {
    [[Enumerable]]: false,
    [[Configurable]]: true,
    [[Set]]: undefined,
  }

  get RegExp.leftContext

  1. Return ? GetLegacyRegExpStaticProperty(%RegExp%, this value, [[RegExpLeftContext]]).
includes: [propertyHelper.js]
features: [legacy-regexp]
---*/

var desc = Object.getOwnPropertyDescriptor(RegExp, "leftContext");

assert.sameValue(desc.set, undefined, "`set` property");
assert.sameValue(typeof desc.get, "function", "`get` property");

verifyProperty(RegExp, "leftContext", {
  enumerable: false,
  configurable: true
});

desc = Object.getOwnPropertyDescriptor(RegExp, "$`");

assert.sameValue(desc.set, undefined, "`set` property");
assert.sameValue(typeof desc.get, "function", "`get` property");

verifyProperty(RegExp, "$`", {
  enumerable: false,
  configurable: true
});

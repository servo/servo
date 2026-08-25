// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Property descriptor for RegExp.$1-$9
info: |
  RegExp.$1-$9 are accessor properties with attributes
  {
    [[Enumerable]]: false,
    [[Configurable]]: true,
    [[Set]]: undefined,
  }

  get RegExp.$1-$9

  1. Return ? GetLegacyRegExpStaticProperty(%RegExp%, this value, [[RegExpParen1-9]]).
includes: [propertyHelper.js]
features: [legacy-regexp]
---*/

for (let i = 1; i <= 9; i++) {
  const property = "$" + i;
  const desc = Object.getOwnPropertyDescriptor(RegExp, property);

  assert.sameValue(desc.set, undefined, property + " setter");
  assert.sameValue(typeof desc.get, "function", property + " getter");

  verifyProperty(RegExp, property, {
    enumerable: false,
    configurable: true
  });
}

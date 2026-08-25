// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: RegExp.$1-$9 throw a TypeError for cross-realm receiver
info: |
  get RegExp.$1-$9

  1. Return ? GetLegacyRegExpStaticProperty(%RegExp%, this value, [[RegExpParen1-9]]).

  GetLegacyRegExpStaticProperty( C, thisValue, internalSlotName ).

  1. Assert C is an object that has an internal slot named internalSlotName.
  2. If SameValue(C, thisValue) is false, throw a TypeError exception.
  3. ...
features: [legacy-regexp,cross-realm,Reflect]
---*/

const other = $262.createRealm().global;

for (let i = 1; i <= 9; i++) {
  const property = "$" + i;
  assert.throws(
    TypeError,
    function () {
      Reflect.get(RegExp, property, other.RegExp);
    },
    "RegExp." + property + " getter throws for cross-realm receiver"
  );
}

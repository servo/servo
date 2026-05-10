// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: RegExp.lastParen throws a TypeError for cross-realm receiver
info: |
  get RegExp.lastParen

  1. Return ? GetLegacyRegExpStaticProperty(%RegExp%, this value, [[RegExpLastParen]]).

  GetLegacyRegExpStaticProperty( C, thisValue, internalSlotName ).

  1. Assert C is an object that has an internal slot named internalSlotName.
  2. If SameValue(C, thisValue) is false, throw a TypeError exception.
  3. ...
features: [legacy-regexp,cross-realm,Reflect]
---*/

const other = $262.createRealm().global;

assert.throws(
  TypeError,
  function () {
    Reflect.get(RegExp, "lastParen", other.RegExp);
  },
  "RegExp.lastParen getter throws for cross-realm receiver"
);

assert.throws(
  TypeError,
  function () {
    Reflect.get(RegExp, "$+", other.RegExp);
  },
  "RegExp.$+ getter throws for cross-realm receiver"
);

// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: RegExp.input throws a TypeError for cross-realm receiver
info: |
  get RegExp.input

  1. Return ? GetLegacyRegExpStaticProperty(%RegExp%, this value, [[RegExpInput]]).

  set RegExp.input = val

  1. Return ? SetLegacyRegExpStaticProperty(%RegExp%, this value, [[RegExpInput]], val).

  GetLegacyRegExpStaticProperty( C, thisValue, internalSlotName ).

  1. Assert C is an object that has an internal slot named internalSlotName.
  2. If SameValue(C, thisValue) is false, throw a TypeError exception.
  3. ...

  SetLegacyRegExpStaticProperty( C, thisValue, internalSlotName, val ).

  1. Assert C is an object that has an internal slot named internalSlotName.
  2. If SameValue(C, thisValue) is false, throw a TypeError exception.
  3. ...
features: [legacy-regexp,cross-realm,Reflect,Reflect.set]
---*/

const other = $262.createRealm().global;

assert.throws(
  TypeError,
  function () {
    Reflect.get(RegExp, "input", other.RegExp);
  },
  "RegExp.input getter throws for cross-realm receiver"
);

assert.throws(
  TypeError,
  function () {
    Reflect.set(RegExp, "input", "", other.RegExp);
  },
  "RegExp.input setter throws for cross-realm receiver"
);

assert.throws(
  TypeError,
  function () {
    Reflect.get(RegExp, "$_", other.RegExp);
  },
  "RegExp.$_ getter throws for cross-realm receiver"
);

assert.throws(
  TypeError,
  function () {
    Reflect.set(RegExp, "$_", "", other.RegExp);
  },
  "RegExp.$_ setter throws for cross-realm receiver"
);

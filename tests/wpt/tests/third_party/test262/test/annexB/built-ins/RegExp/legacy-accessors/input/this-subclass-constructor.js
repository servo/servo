// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: RegExp.input throws a TypeError for subclass receiver
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
features: [legacy-regexp,class]
---*/

class MyRegExp extends RegExp {}

assert.throws(
  TypeError,
  function () {
    MyRegExp.input;
  },
  "RegExp.input getter throws for subclass receiver"
);

assert.throws(
  TypeError,
  function () {
    MyRegExp.input = "";
  },
  "RegExp.input setter throws for subclass receiver"
);

assert.throws(
  TypeError,
  function () {
    MyRegExp.$_;
  },
  "RegExp.$_ getter throws for subclass receiver"
);

assert.throws(
  TypeError,
  function () {
    MyRegExp.$_ = "";
  },
  "RegExp.$_ setter throws for subclass receiver"
);

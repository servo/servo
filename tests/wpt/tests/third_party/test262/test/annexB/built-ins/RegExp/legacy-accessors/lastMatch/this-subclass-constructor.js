// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: RegExp.lastMatch throws a TypeError for subclass receiver
info: |
  get RegExp.lastMatch

  1. Return ? GetLegacyRegExpStaticProperty(%RegExp%, this value, [[RegExpLastMatch]]).

  GetLegacyRegExpStaticProperty( C, thisValue, internalSlotName ).

  1. Assert C is an object that has an internal slot named internalSlotName.
  2. If SameValue(C, thisValue) is false, throw a TypeError exception.
  3. ...
features: [legacy-regexp,class]
---*/

class MyRegExp extends RegExp {}

assert.throws(
  TypeError,
  function () {
    MyRegExp.lastMatch;
  },
  "RegExp.lastMatch getter throws for subclass receiver"
);

assert.throws(
  TypeError,
  function () {
    MyRegExp["$&"];
  },
  "RegExp.$& getter throws for subclass receiver"
);

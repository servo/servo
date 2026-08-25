// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-Intl.DisplayNames
description: Default [[Prototype]] value derived from realm of the newTarget
info: |
  Intl.DisplayNames ( locales , options )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let displayNames be ? OrdinaryCreateFromConstructor(NewTarget, "%DisplayNamesPrototype%",
    « [[InitializedDisplayNames]], [[Locale]], [[Style]], [[Type]], [[Fallback]], [[Fields]] »).
  ...
  12. Let type be ? GetOption(options, "type", "string", « "language", "region", "script", "currency" », undefined).
  13. If type is undefined, throw a TypeError exception.

  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ , internalSlotsList ] )

  ...
  2. Let proto be ? GetPrototypeFromConstructor(constructor, intrinsicDefaultProto).
  3. Return ObjectCreate(proto, internalSlotsList).

  GetPrototypeFromConstructor ( constructor, intrinsicDefaultProto )

  ...
  3. Let proto be ? Get(constructor, "prototype").
  4. If Type(proto) is not Object, then
    a. Let realm be ? GetFunctionRealm(constructor).
    b. Set proto to realm's intrinsic object named intrinsicDefaultProto.
  5. Return proto.
features: [Intl.DisplayNames, cross-realm, Reflect]
---*/

var other = $262.createRealm().global;
var C = new other.Function();
C.prototype = null;

var o = Reflect.construct(Intl.DisplayNames, [undefined, {type: 'language'}], C);

assert.sameValue(Object.getPrototypeOf(o), other.Intl.DisplayNames.prototype);

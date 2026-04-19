// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: Default [[Prototype]] value derived from realm of the NewTarget.
info: |
  Intl.RelativeTimeFormat ([ locales [ , options ]])

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let relativeTimeFormat be ? OrdinaryCreateFromConstructor(NewTarget, "%RelativeTimeFormatPrototype%", « ... »).
  3. Return ? InitializeRelativeTimeFormat(relativeTimeFormat, locales, options).

  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ , internalSlotsList ] )

  ...
  2. Let proto be ? GetPrototypeFromConstructor(constructor, intrinsicDefaultProto).
  3. Return ObjectCreate(proto, internalSlotsList).

  GetPrototypeFromConstructor ( constructor, intrinsicDefaultProto )

  ...
  3. Let proto be ? Get(constructor, 'prototype').
  4. If Type(proto) is not Object, then
    a. Let realm be ? GetFunctionRealm(constructor).
    b. Set proto to realm's intrinsic object named intrinsicDefaultProto.
  5. Return proto.
features: [Intl.RelativeTimeFormat, cross-realm, Reflect, Symbol]
---*/

var other = $262.createRealm().global;
var newTarget = new other.Function();
var rtf;

newTarget.prototype = undefined;
rtf = Reflect.construct(Intl.RelativeTimeFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(rtf), other.Intl.RelativeTimeFormat.prototype, 'newTarget.prototype is undefined');

newTarget.prototype = null;
rtf = Reflect.construct(Intl.RelativeTimeFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(rtf), other.Intl.RelativeTimeFormat.prototype, 'newTarget.prototype is null');

newTarget.prototype = true;
rtf = Reflect.construct(Intl.RelativeTimeFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(rtf), other.Intl.RelativeTimeFormat.prototype, 'newTarget.prototype is a Boolean');

newTarget.prototype = '';
rtf = Reflect.construct(Intl.RelativeTimeFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(rtf), other.Intl.RelativeTimeFormat.prototype, 'newTarget.prototype is a String');

newTarget.prototype = Symbol();
rtf = Reflect.construct(Intl.RelativeTimeFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(rtf), other.Intl.RelativeTimeFormat.prototype, 'newTarget.prototype is a Symbol');

newTarget.prototype = -1;
rtf = Reflect.construct(Intl.RelativeTimeFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(rtf), other.Intl.RelativeTimeFormat.prototype, 'newTarget.prototype is a Number');

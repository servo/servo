// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat
description: Default [[Prototype]] value derived from realm of the NewTarget.
info: |
  Intl.ListFormat ( [ locales [ , options ] ] )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let listFormat be ? OrdinaryCreateFromConstructor(NewTarget, "%ListFormatPrototype%", « ... »).
  ...
  24. Return listFormat.

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
features: [Intl.ListFormat, cross-realm, Reflect, Symbol]
---*/

var other = $262.createRealm().global;
var newTarget = new other.Function();
var lf;

newTarget.prototype = undefined;
lf = Reflect.construct(Intl.ListFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(lf), other.Intl.ListFormat.prototype, 'newTarget.prototype is undefined');

newTarget.prototype = null;
lf = Reflect.construct(Intl.ListFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(lf), other.Intl.ListFormat.prototype, 'newTarget.prototype is null');

newTarget.prototype = true;
lf = Reflect.construct(Intl.ListFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(lf), other.Intl.ListFormat.prototype, 'newTarget.prototype is a Boolean');

newTarget.prototype = 'str';
lf = Reflect.construct(Intl.ListFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(lf), other.Intl.ListFormat.prototype, 'newTarget.prototype is a String');

newTarget.prototype = Symbol();
lf = Reflect.construct(Intl.ListFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(lf), other.Intl.ListFormat.prototype, 'newTarget.prototype is a Symbol');

newTarget.prototype = NaN;
lf = Reflect.construct(Intl.ListFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(lf), other.Intl.ListFormat.prototype, 'newTarget.prototype is a Number');

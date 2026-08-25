// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat
description: Default [[Prototype]] value derived from realm of the NewTarget.
info: |
  Intl.DateTimeFormat ( [ locales [ , options ] ] )

  1. If NewTarget is undefined, let newTarget be the active function object, else let newTarget be NewTarget.
  2. Let dateTimeFormat be ? OrdinaryCreateFromConstructor(newTarget, "%DateTimeFormatPrototype%", « ... »).
  ...
  6. Return dateTimeFormat.

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
features: [cross-realm, Reflect, Symbol]
---*/

var other = $262.createRealm().global;
var newTarget = new other.Function();
var dtf;

newTarget.prototype = undefined;
dtf = Reflect.construct(Intl.DateTimeFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(dtf), other.Intl.DateTimeFormat.prototype, 'newTarget.prototype is undefined');

newTarget.prototype = null;
dtf = Reflect.construct(Intl.DateTimeFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(dtf), other.Intl.DateTimeFormat.prototype, 'newTarget.prototype is null');

newTarget.prototype = false;
dtf = Reflect.construct(Intl.DateTimeFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(dtf), other.Intl.DateTimeFormat.prototype, 'newTarget.prototype is a Boolean');

newTarget.prototype = 'str';
dtf = Reflect.construct(Intl.DateTimeFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(dtf), other.Intl.DateTimeFormat.prototype, 'newTarget.prototype is a String');

newTarget.prototype = Symbol();
dtf = Reflect.construct(Intl.DateTimeFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(dtf), other.Intl.DateTimeFormat.prototype, 'newTarget.prototype is a Symbol');

newTarget.prototype = 1;
dtf = Reflect.construct(Intl.DateTimeFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(dtf), other.Intl.DateTimeFormat.prototype, 'newTarget.prototype is a Number');

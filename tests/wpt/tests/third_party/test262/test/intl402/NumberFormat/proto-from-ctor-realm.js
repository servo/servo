// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat
description: Default [[Prototype]] value derived from realm of the NewTarget.
info: |
  Intl.NumberFormat ( [ locales [ , options ] ] )

  1. If NewTarget is undefined, let newTarget be the active function object, else let newTarget be NewTarget.
  2. Let numberFormat be ? OrdinaryCreateFromConstructor(newTarget, "%NumberFormatPrototype%", « ... »).
  ...
  6. Return numberFormat.

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
var nf;

newTarget.prototype = undefined;
nf = Reflect.construct(Intl.NumberFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(nf), other.Intl.NumberFormat.prototype, 'newTarget.prototype is undefined');

newTarget.prototype = null;
nf = Reflect.construct(Intl.NumberFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(nf), other.Intl.NumberFormat.prototype, 'newTarget.prototype is null');

newTarget.prototype = true;
nf = Reflect.construct(Intl.NumberFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(nf), other.Intl.NumberFormat.prototype, 'newTarget.prototype is a Boolean');

newTarget.prototype = 'str';
nf = Reflect.construct(Intl.NumberFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(nf), other.Intl.NumberFormat.prototype, 'newTarget.prototype is a String');

newTarget.prototype = Symbol();
nf = Reflect.construct(Intl.NumberFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(nf), other.Intl.NumberFormat.prototype, 'newTarget.prototype is a Symbol');

newTarget.prototype = 0;
nf = Reflect.construct(Intl.NumberFormat, [], newTarget);
assert.sameValue(Object.getPrototypeOf(nf), other.Intl.NumberFormat.prototype, 'newTarget.prototype is a Number');

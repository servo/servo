// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale
description: Default [[Prototype]] value derived from realm of the NewTarget.
info: |
  Intl.Locale ( tag [ , options] )

  ...
  6. Let locale be ? OrdinaryCreateFromConstructor(NewTarget, %LocalePrototype%, internalSlotsList).
  ...
  38. Return locale.

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
features: [Intl.Locale, cross-realm, Reflect, Symbol]
---*/

var other = $262.createRealm().global;
var newTarget = new other.Function();
var locale;

newTarget.prototype = undefined;
locale = Reflect.construct(Intl.Locale, ['de'], newTarget);
assert.sameValue(Object.getPrototypeOf(locale), other.Intl.Locale.prototype, 'newTarget.prototype is undefined');

newTarget.prototype = null;
locale = Reflect.construct(Intl.Locale, ['de'], newTarget);
assert.sameValue(Object.getPrototypeOf(locale), other.Intl.Locale.prototype, 'newTarget.prototype is null');

newTarget.prototype = true;
locale = Reflect.construct(Intl.Locale, ['de'], newTarget);
assert.sameValue(Object.getPrototypeOf(locale), other.Intl.Locale.prototype, 'newTarget.prototype is a Boolean');

newTarget.prototype = 'str';
locale = Reflect.construct(Intl.Locale, ['de'], newTarget);
assert.sameValue(Object.getPrototypeOf(locale), other.Intl.Locale.prototype, 'newTarget.prototype is a String');

newTarget.prototype = Symbol();
locale = Reflect.construct(Intl.Locale, ['de'], newTarget);
assert.sameValue(Object.getPrototypeOf(locale), other.Intl.Locale.prototype, 'newTarget.prototype is a Symbol');

newTarget.prototype = 0;
locale = Reflect.construct(Intl.Locale, ['de'], newTarget);
assert.sameValue(Object.getPrototypeOf(locale), other.Intl.Locale.prototype, 'newTarget.prototype is a Number');

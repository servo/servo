// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.pluralrules
description: Default [[Prototype]] value derived from realm of the NewTarget.
info: |
  Intl.PluralRules ( [ locales [ , options ] ] )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let pluralRules be ? OrdinaryCreateFromConstructor(newTarget, "%PluralRulesPrototype%", « ... »).
  3. Return ? InitializePluralRules(pluralRules, locales, options).

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
var pr;

newTarget.prototype = undefined;
pr = Reflect.construct(Intl.PluralRules, [], newTarget);
assert.sameValue(Object.getPrototypeOf(pr), other.Intl.PluralRules.prototype, 'newTarget.prototype is undefined');

newTarget.prototype = null;
pr = Reflect.construct(Intl.PluralRules, [], newTarget);
assert.sameValue(Object.getPrototypeOf(pr), other.Intl.PluralRules.prototype, 'newTarget.prototype is null');

newTarget.prototype = false;
pr = Reflect.construct(Intl.PluralRules, [], newTarget);
assert.sameValue(Object.getPrototypeOf(pr), other.Intl.PluralRules.prototype, 'newTarget.prototype is a Boolean');

newTarget.prototype = '';
pr = Reflect.construct(Intl.PluralRules, [], newTarget);
assert.sameValue(Object.getPrototypeOf(pr), other.Intl.PluralRules.prototype, 'newTarget.prototype is a String');

newTarget.prototype = Symbol();
pr = Reflect.construct(Intl.PluralRules, [], newTarget);
assert.sameValue(Object.getPrototypeOf(pr), other.Intl.PluralRules.prototype, 'newTarget.prototype is a Symbol');

newTarget.prototype = 0;
pr = Reflect.construct(Intl.PluralRules, [], newTarget);
assert.sameValue(Object.getPrototypeOf(pr), other.Intl.PluralRules.prototype, 'newTarget.prototype is a Number');

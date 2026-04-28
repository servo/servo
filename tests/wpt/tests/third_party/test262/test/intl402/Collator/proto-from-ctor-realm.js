// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.collator
description: Default [[Prototype]] value derived from realm of the NewTarget.
info: |
  Intl.Collator ( [ locales [ , options ] ] )

  1. If NewTarget is undefined, let newTarget be the active function object, else let newTarget be NewTarget.
  ...
  5. Let collator be ? OrdinaryCreateFromConstructor(newTarget, "%CollatorPrototype%", internalSlotsList).
  6. Return ? InitializeCollator(collator, locales, options).

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
var col;

newTarget.prototype = undefined;
col = Reflect.construct(Intl.Collator, [], newTarget);
assert.sameValue(Object.getPrototypeOf(col), other.Intl.Collator.prototype, 'newTarget.prototype is undefined');

newTarget.prototype = null;
col = Reflect.construct(Intl.Collator, [], newTarget);
assert.sameValue(Object.getPrototypeOf(col), other.Intl.Collator.prototype, 'newTarget.prototype is null');

newTarget.prototype = true;
col = Reflect.construct(Intl.Collator, [], newTarget);
assert.sameValue(Object.getPrototypeOf(col), other.Intl.Collator.prototype, 'newTarget.prototype is a Boolean');

newTarget.prototype = '';
col = Reflect.construct(Intl.Collator, [], newTarget);
assert.sameValue(Object.getPrototypeOf(col), other.Intl.Collator.prototype, 'newTarget.prototype is a String');

newTarget.prototype = Symbol();
col = Reflect.construct(Intl.Collator, [], newTarget);
assert.sameValue(Object.getPrototypeOf(col), other.Intl.Collator.prototype, 'newTarget.prototype is a Symbol');

newTarget.prototype = 1;
col = Reflect.construct(Intl.Collator, [], newTarget);
assert.sameValue(Object.getPrototypeOf(col), other.Intl.Collator.prototype, 'newTarget.prototype is a Number');

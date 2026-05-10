// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.segmenter
description: Default [[Prototype]] value derived from realm of the NewTarget.
info: |
  Intl.Segmenter ([ locales [ , options ]])
  1. If NewTarget is undefined, throw a TypeError exception.
  3. Let segmenter be ? OrdinaryCreateFromConstructor(NewTarget, "%Segmenter.prototype%", internalSlotsList).
  ...
  15. Return segmenter.
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
features: [cross-realm, Reflect, Symbol, Intl.Segmenter]
---*/

const other = $262.createRealm().global;
const newTarget = new other.Function();
let sgm;

newTarget.prototype = undefined;
sgm = Reflect.construct(Intl.Segmenter, [], newTarget);
assert.sameValue(Object.getPrototypeOf(sgm), other.Intl.Segmenter.prototype, 'newTarget.prototype is undefined');

newTarget.prototype = null;
sgm = Reflect.construct(Intl.Segmenter, [], newTarget);
assert.sameValue(Object.getPrototypeOf(sgm), other.Intl.Segmenter.prototype, 'newTarget.prototype is null');

newTarget.prototype = false;
sgm = Reflect.construct(Intl.Segmenter, [], newTarget);
assert.sameValue(Object.getPrototypeOf(sgm), other.Intl.Segmenter.prototype, 'newTarget.prototype is a Boolean');

newTarget.prototype = 'str';
sgm = Reflect.construct(Intl.Segmenter, [], newTarget);
assert.sameValue(Object.getPrototypeOf(sgm), other.Intl.Segmenter.prototype, 'newTarget.prototype is a String');

newTarget.prototype = Symbol();
sgm = Reflect.construct(Intl.Segmenter, [], newTarget);
assert.sameValue(Object.getPrototypeOf(sgm), other.Intl.Segmenter.prototype, 'newTarget.prototype is a Symbol');

newTarget.prototype = 1;
sgm = Reflect.construct(Intl.Segmenter, [], newTarget);
assert.sameValue(Object.getPrototypeOf(sgm), other.Intl.Segmenter.prototype, 'newTarget.prototype is a Number');

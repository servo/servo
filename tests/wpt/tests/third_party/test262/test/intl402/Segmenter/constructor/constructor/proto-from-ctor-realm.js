// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter
description: Default [[Prototype]] value derived from realm of the NewTarget.
info: |
  Intl.Segmenter ([ locales [ , options ]])

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let segmenter be ? OrdinaryCreateFromConstructor(NewTarget, "%SegmenterPrototype%", « [[InitializedSegmenter]] »).
  ...
  14. Return segmenter.

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
features: [Intl.Segmenter, cross-realm, Reflect, Symbol]
---*/

var other = $262.createRealm().global;
var newTarget = new other.Function();
var seg;

newTarget.prototype = undefined;
seg = Reflect.construct(Intl.Segmenter, [], newTarget);
assert.sameValue(Object.getPrototypeOf(seg), other.Intl.Segmenter.prototype, 'newTarget.prototype is undefined');

newTarget.prototype = null;
seg = Reflect.construct(Intl.Segmenter, [], newTarget);
assert.sameValue(Object.getPrototypeOf(seg), other.Intl.Segmenter.prototype, 'newTarget.prototype is null');

newTarget.prototype = false;
seg = Reflect.construct(Intl.Segmenter, [], newTarget);
assert.sameValue(Object.getPrototypeOf(seg), other.Intl.Segmenter.prototype, 'newTarget.prototype is a Boolean');

newTarget.prototype = 'str';
seg = Reflect.construct(Intl.Segmenter, [], newTarget);
assert.sameValue(Object.getPrototypeOf(seg), other.Intl.Segmenter.prototype, 'newTarget.prototype is a String');

newTarget.prototype = Symbol();
seg = Reflect.construct(Intl.Segmenter, [], newTarget);
assert.sameValue(Object.getPrototypeOf(seg), other.Intl.Segmenter.prototype, 'newTarget.prototype is a Symbol');

newTarget.prototype = 1;
seg = Reflect.construct(Intl.Segmenter, [], newTarget);
assert.sameValue(Object.getPrototypeOf(seg), other.Intl.Segmenter.prototype, 'newTarget.prototype is a Number');

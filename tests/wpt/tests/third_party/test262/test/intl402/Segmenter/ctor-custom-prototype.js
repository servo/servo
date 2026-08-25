// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.segmenter
description: >
  Custom Prototype of the returned object based on the NewTarget
info: |
  Intl.Segmenter ([ locales [ , options ]])
  1. If NewTarget is undefined, throw a TypeError exception.
  3. Let segmenter be ? OrdinaryCreateFromConstructor(NewTarget, "%Segmenter.prototype%", internalSlotsList).
  ...
  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ , internalSlotsList ] )
  ...
  2. Let proto be ? GetPrototypeFromConstructor(constructor, intrinsicDefaultProto).
  ...
  GetPrototypeFromConstructor ( constructor, intrinsicDefaultProto )
  3. Let proto be ? Get(constructor, "prototype").
  4. If Type(proto) is not Object, then
    a. Let realm be ? GetFunctionRealm(constructor).
    b. Set proto to realm's intrinsic object named intrinsicDefaultProto.
  5. Return proto.
features: [Intl.Segmenter, Reflect]
---*/

var custom = new Function();
custom.prototype = {};

const obj = Reflect.construct(Intl.Segmenter, [], custom);

assert.sameValue(Object.getPrototypeOf(obj), custom.prototype);

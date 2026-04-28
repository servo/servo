// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.segmenter
description: >
  Return abrupt from Get Prototype from a custom NewTarget
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
features: [Intl.Segmenter, Reflect, Proxy]
---*/

const custom = new Proxy(new Function(), {
  get(target, key) {
    if (key === 'prototype') {
      throw new Test262Error();
    }

    return target[key];
  }
});

assert.throws(Test262Error, () => {
  Reflect.construct(Intl.Segmenter, [], custom);
});

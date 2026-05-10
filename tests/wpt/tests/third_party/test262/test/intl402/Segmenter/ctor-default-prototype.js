// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.segmenter
description: >
  Prototype of the returned object is Segmenter.prototype
info: |
  Intl.Segmenter ([ locales [ , options ]])
  1. If NewTarget is undefined, throw a TypeError exception.
  3. Let segmenter be ? OrdinaryCreateFromConstructor(NewTarget, "%Segmenter.prototype%", internalSlotsList).
features: [Intl.Segmenter]
---*/

var obj = new Intl.Segmenter();

assert.sameValue(Object.getPrototypeOf(obj), Intl.Segmenter.prototype);

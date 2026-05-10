// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.Segmenter
description: >
  Throws TypeError if options is null
info: |
  Intl.Segmenter ([ locales [ , options ]])
  1. If NewTarget is undefined, throw a TypeError exception.
  3. Let segmenter be ? OrdinaryCreateFromConstructor(NewTarget, "%Segmenter.prototype%", internalSlotsList).
  ...
  4. If options is undefined, then
    a. Let options be ObjectCreate(null).
  5. Else
    a. Let options be ? ToObject(options).
  ...
features: [Intl.Segmenter]
---*/

assert.throws(TypeError, () => {
  new Intl.Segmenter(undefined, null);
}, 'null');

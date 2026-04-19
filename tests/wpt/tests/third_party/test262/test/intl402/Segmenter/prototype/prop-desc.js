// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter.prototype
description: Checks the "prototype" property of the Segmenter constructor.
info: |
    Intl.Segmenter.prototype

    The value of Intl.Segmenter.prototype is %SegmenterPrototype%.

    This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: false }.
includes: [propertyHelper.js]
features: [Intl.Segmenter]
---*/

verifyProperty(Intl.Segmenter, "prototype", {
  writable: false,
  enumerable: false,
  configurable: false,
});

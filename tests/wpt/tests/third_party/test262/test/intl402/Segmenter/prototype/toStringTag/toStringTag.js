// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.Segmenter.prototype-@@tostringtag
description: >
    Checks the @@toStringTag property of the Segmenter prototype object.
info: |
    Intl.Segmenter.prototype[ @@toStringTag ]

    The initial value of the @@toStringTag property is the string value "Intl.Segmenter".

    This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Intl.Segmenter, Symbol.toStringTag]
---*/

verifyProperty(Intl.Segmenter.prototype, Symbol.toStringTag, {
  value: "Intl.Segmenter",
  writable: false,
  enumerable: false,
  configurable: true,
});

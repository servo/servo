// Copyright 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.segmenter.prototype-@@tostringtag
description: >
  Property descriptor of Segmenter.prototype[@@toStringTag]
info: |
  The initial value of the @@toStringTag property is the string value "Intl.Segmenter".
  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Intl.Segmenter, Symbol.toStringTag]
---*/

verifyProperty(Intl.Segmenter.prototype, Symbol.toStringTag, {
  value: "Intl.Segmenter",
  writable: false,
  enumerable: false,
  configurable: true
});

// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry-@@tostringtag
description: >
    `Symbol.toStringTag` property descriptor
info: |
    The initial value of the @@toStringTag property is the String value
    'FinalizationRegistry'.

    This property has the attributes { [[Writable]]: false, [[Enumerable]]:
    false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [FinalizationRegistry, Symbol, Symbol.toStringTag]
---*/

verifyProperty(FinalizationRegistry.prototype, Symbol.toStringTag, {
  value: 'FinalizationRegistry',
  writable: false,
  enumerable: false,
  configurable: true
});

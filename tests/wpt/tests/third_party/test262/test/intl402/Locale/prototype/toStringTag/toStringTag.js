// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype-@@tostringtag
description: >
    Checks the @@toStringTag property of the Locale prototype object.
info: |
    Intl.Locale.prototype[ @@toStringTag ]

    The initial value of the @@toStringTag property is the string value "Intl.Locale".

    This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Intl.Locale, Symbol.toStringTag]
---*/

verifyProperty(Intl.Locale.prototype, Symbol.toStringTag, {
  value: 'Intl.Locale',
  writable: false,
  enumerable: false,
  configurable: true
});

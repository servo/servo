// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.ListFormat.prototype-@@tostringtag
description: >
    Checks the @@toStringTag property of the ListFormat prototype object.
info: |
    Intl.ListFormat.prototype[ @@toStringTag ]

    The initial value of the @@toStringTag property is the string value "Intl.ListFormat".

    This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Intl.ListFormat, Symbol.toStringTag]
---*/

verifyProperty(Intl.ListFormat.prototype, Symbol.toStringTag, {
  value: "Intl.ListFormat",
  writable: false,
  enumerable: false,
  configurable: true,
});

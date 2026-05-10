// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.RelativeTimeFormat.prototype-@@tostringtag
description: >
    Checks the @@toStringTag property of the RelativeTimeFormat prototype object.
info: |
    Intl.RelativeTimeFormat.prototype[ @@toStringTag ]

    The initial value of the @@toStringTag property is the string value "Intl.RelativeTimeFormat".

    This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Intl.RelativeTimeFormat, Symbol.toStringTag]
---*/

verifyProperty(Intl.RelativeTimeFormat.prototype, Symbol.toStringTag, {
  value: "Intl.RelativeTimeFormat",
  writable: false,
  enumerable: false,
  configurable: true
});

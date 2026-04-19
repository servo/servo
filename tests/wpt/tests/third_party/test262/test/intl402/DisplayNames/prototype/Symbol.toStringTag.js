// Copyright 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.DisplayNames.prototype-@@tostringtag
description: >
  Property descriptor of DisplayNames.prototype[@@toStringTag]
info: |
  The initial value of the @@toStringTag property is the string value "Intl.DisplayNames".

  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Intl.DisplayNames, Symbol.toStringTag]
---*/

verifyProperty(Intl.DisplayNames.prototype, Symbol.toStringTag, {
  value: "Intl.DisplayNames",
  writable: false,
  enumerable: false,
  configurable: true
});

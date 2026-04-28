// Copyright (C) 2021 Nikhil Singhal. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype
description: Prototype attributes verification
info: |
  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: false }.
features: [Intl.DurationFormat]
includes: [propertyHelper.js]
---*/

verifyProperty(Intl.DurationFormat, "prototype", {
  value: Intl.DurationFormat.prototype,
  writable: false,
  enumerable: false,
  configurable: false,
});

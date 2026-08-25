// Copyright (C) 2021 Nikhil Singhal. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
// Copyright 2022 Igalia, S.L. All rights reserved.

/*---
esid: sec-intl.DurationFormat.prototype-@@tostringtag
description: >
  Property descriptor of Intl.DurationFormat.prototype[@@toStringTag].
info: |
  Intl.DurationFormat.prototype [ @@toStringTag ]

  The initial value of the @@toStringTag property is the string value "Intl.DurationFormat".

  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.

features: [Intl.DurationFormat, Symbol.toStringTag]
includes: [propertyHelper.js]
---*/

verifyProperty(Intl.DurationFormat.prototype, Symbol.toStringTag, {
  value: "Intl.DurationFormat",
  writable: false,
  enumerable: false,
  configurable: true
});

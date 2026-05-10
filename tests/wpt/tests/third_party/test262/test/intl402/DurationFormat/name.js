// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat
description: >
  Intl.DurationFormat.name is "DurationFormat".
info: |
  Intl.DurationFormat ([ locales [ , options ]])

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.

features: [Intl.DurationFormat]
includes: [propertyHelper.js]
---*/

verifyProperty(Intl.DurationFormat, 'name', {
  value: 'DurationFormat',
  enumerable: false,
  writable: false,
  configurable: true
});

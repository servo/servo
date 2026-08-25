// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat.prototype.formatToParts
description: >
  Intl.DateTimeFormat.prototype.formatToParts.length is 1.
info: |
  Intl.DateTimeFormat.prototype.formatToParts ( date )

  17 ECMAScript Standard Built-in Objects:

    Every built-in function object, including constructors, has a length
    property whose value is an integer. Unless otherwise specified, this
    value is equal to the largest number of named arguments shown in the
    subclause headings for the function description. Optional parameters
    (which are indicated with brackets: [ ]) or rest parameters (which
    are shown using the form «...name») are not included in the default
    argument count.
    Unless otherwise specified, the length property of a built-in function
    object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: true }.

features: [Intl.DurationFormat]
includes: [propertyHelper.js]
---*/

assert.sameValue(Intl.DateTimeFormat.prototype.formatToParts.length, 1);

verifyProperty(Intl.DurationFormat.prototype.formatToParts, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});


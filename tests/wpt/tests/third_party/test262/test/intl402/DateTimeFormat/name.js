// Copyright (C) 2016 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DateTimeFormat
description: >
  Intl.DateTimeFormat.name is "DateTimeFormat".
info: |
  12.2.1 Intl.DateTimeFormat ([ locales [ , options ]])

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

verifyProperty(Intl.DateTimeFormat, "name", {
  value: "DateTimeFormat",
  writable: false,
  enumerable: false,
  configurable: true,
});

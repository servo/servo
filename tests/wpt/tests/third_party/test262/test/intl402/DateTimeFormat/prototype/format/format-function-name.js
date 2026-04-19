// Copyright (C) 2016 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DateTimeFormat.prototype.format
description: >
  The bound DateTimeFormat format function is an anonymous function.
info: |
  12.4.3 get Intl.DateTimeFormat.prototype.compare

  17 ECMAScript Standard Built-in Objects:
    Every built-in function object, including constructors, has a `name`
    property whose value is a String. Functions that are identified as
    anonymous functions use the empty string as the value of the `name`
    property.
    Unless otherwise specified, the `name` property of a built-in function
    object has the attributes { [[Writable]]: *false*, [[Enumerable]]: *false*,
    [[Configurable]]: *true* }.
includes: [propertyHelper.js]
---*/

var formatFn = new Intl.DateTimeFormat().format;

verifyProperty(formatFn, "name", {
  value: "", writable: false, enumerable: false, configurable: true
});

// Copyright (C) 2016 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Collator.prototype.compare
description: >
  The bound Collator compare function is an anonymous function.
info: |
  10.3.3 get Intl.Collator.prototype.compare

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

var compareFn = new Intl.Collator().compare;

verifyProperty(compareFn, "name", {
  value: "", writable: false, enumerable: false, configurable: true
});

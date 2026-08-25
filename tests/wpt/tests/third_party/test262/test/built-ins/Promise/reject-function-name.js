// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise-reject-functions
description: The `name` property of Promise Reject functions
info: |
  A promise reject function is an anonymous built-in function.

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

var rejectFunction;
new Promise(function(resolve, reject) {
  rejectFunction = reject;
});

verifyProperty(rejectFunction, "name", {
  value: "", writable: false, enumerable: false, configurable: true
});

// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-getcapabilitiesexecutor-functions
description: The `name` property of GetCapabilitiesExecutor functions
info: |
  A GetCapabilitiesExecutor function is an anonymous built-in function.

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

var executorFunction;

function NotPromise(executor) {
  executorFunction = executor;
  executor(function() {}, function() {});
}
Promise.resolve.call(NotPromise);

verifyProperty(executorFunction, "name", {
  value: "", writable: false, enumerable: false, configurable: true
});

// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: sec-async-function-constructor-properties
description: >
  %AsyncFunction% has a name of "AsyncFunction".
includes: [propertyHelper.js]
---*/

var AsyncFunction = async function foo() {}.constructor;
verifyProperty(AsyncFunction, "name", {
  value: "AsyncFunction",
  writable: false,
  enumerable: false,
  configurable: true
});

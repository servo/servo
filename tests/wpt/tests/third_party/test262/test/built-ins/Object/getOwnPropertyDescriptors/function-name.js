// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Object.getOwnPropertyDescriptors should have name property with value 'getOwnPropertyDescriptors'
esid: sec-object.getownpropertydescriptors
author: Jordan Harband
includes: [propertyHelper.js]
---*/

verifyProperty(Object.getOwnPropertyDescriptors, "name", {
  value: "getOwnPropertyDescriptors",
  writable: false,
  enumerable: false,
  configurable: true,
});

// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Object.getOwnPropertyDescriptors should have length 1
esid: sec-object.getownpropertydescriptors
author: Jordan Harband
includes: [propertyHelper.js]
---*/

verifyProperty(Object.getOwnPropertyDescriptors, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true,
});

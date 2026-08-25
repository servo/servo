// Copyright (C) 2024 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Promise.try `length` property
includes: [propertyHelper.js]
features: [promise-try]
---*/

verifyProperty(Promise.try, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});

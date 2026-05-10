// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Object.getOwnPropertyDescriptors should produce a normal object inheriting from Object.prototype
esid: sec-object.getownpropertydescriptors
author: Jordan Harband
---*/

assert.sameValue(
  Object.getPrototypeOf(Object.getOwnPropertyDescriptors({})),
  Object.prototype
);

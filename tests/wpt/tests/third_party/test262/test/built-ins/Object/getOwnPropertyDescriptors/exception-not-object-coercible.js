// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Object.getOwnPropertyDescriptors should fail if given a null or undefined value
esid: sec-object.getownpropertydescriptors
author: Jordan Harband
---*/

assert.throws(TypeError, function() {
  Object.getOwnPropertyDescriptors(null);
});

assert.throws(TypeError, function() {
  Object.getOwnPropertyDescriptors(undefined);
});

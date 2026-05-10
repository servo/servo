// Copyright (C) 2015 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.entries
description: Object.entries should fail if given a null or undefined value
author: Jordan Harband
---*/

assert.throws(TypeError, function() {
  Object.entries(null);
});

assert.throws(TypeError, function() {
  Object.entries(undefined);
});

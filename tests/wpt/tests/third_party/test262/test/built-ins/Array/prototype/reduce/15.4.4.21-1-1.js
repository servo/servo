// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: Array.prototype.reduce applied to undefined
---*/


assert.throws(TypeError, function() {
  Array.prototype.reduce.call(undefined);
});

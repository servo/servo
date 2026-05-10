// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.toisostring
description: >
    Date.prototype.toISOString - TypeError is thrown when this is any
    other objects instead of Date object
---*/


assert.throws(TypeError, function() {
  Date.prototype.toISOString.call([]);
});

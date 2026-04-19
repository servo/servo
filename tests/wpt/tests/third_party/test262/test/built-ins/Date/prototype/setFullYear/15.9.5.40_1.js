// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.setfullyear
description: >
    Date.prototype.setFullYear - Date.prototype is itself not an
    instance of Date
---*/


assert.throws(TypeError, function() {
  Date.prototype.setFullYear(2012);
});

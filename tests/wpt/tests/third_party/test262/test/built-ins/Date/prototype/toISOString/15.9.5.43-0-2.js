// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.toisostring
description: >
    Date.prototype.toISOString must exist as a function taking 0
    parameters
---*/

assert.sameValue(Date.prototype.toISOString.length, 0, 'Date.prototype.toISOString.length');

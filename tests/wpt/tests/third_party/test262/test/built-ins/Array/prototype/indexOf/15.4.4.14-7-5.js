// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf returns correct index when 'fromIndex' is 1
---*/

assert.sameValue([1, 2, 3].indexOf(2, 1), 1, '[1, 2, 3].indexOf(2, 1)');

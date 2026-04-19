// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-1-4
description: String.prototype.trim works for primitive type number
---*/

assert.sameValue(String.prototype.trim.call(0), "0", 'String.prototype.trim.call(0)');

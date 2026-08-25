// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-2-34
description: >
    String.prototype.trim - 'this' is an array that converts to a
    string
---*/

assert.sameValue(String.prototype.trim.call([1]), '1', 'String.prototype.trim.call([1])');

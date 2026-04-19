// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - both array element and search
    element are strings, and they have exactly the same sequence of
    characters
---*/

assert.sameValue(["abc", "ab", "bca", ""].lastIndexOf("abc"), 0, '["abc", "ab", "bca", ""].lastIndexOf("abc")');

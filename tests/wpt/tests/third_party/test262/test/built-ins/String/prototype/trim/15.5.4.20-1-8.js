// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-1-8
description: >
    String.prototype.trim works for a primitive string (value is '
    abc')
---*/

var strObj = String("    abc");

assert.sameValue(strObj.trim(), "abc", 'strObj.trim()');
assert.sameValue(strObj.toString(), "    abc", 'strObj.toString()');

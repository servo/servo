// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-2-45
description: >
    String.prototype.trim - 'this' is a string that contains white
    space, character, number, object and null characters
---*/

var str = "abc" + "   " + 123 + "   " + {} + "    " + "\u0000";
var str1 = "    " + str + "    ";

assert.sameValue(str1.trim(), str, 'str1.trim()');

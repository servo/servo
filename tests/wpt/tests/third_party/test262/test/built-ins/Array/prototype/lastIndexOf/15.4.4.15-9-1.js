// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf returns -1 for elements not present
---*/

var a = new Array();
a[100] = 1;
a[99999] = "";
a[10] = new Object();
a[5555] = 5.5;
a[123456] = "str";
a[5] = 1E+309;

assert.sameValue(a.lastIndexOf(1), 100, 'a.lastIndexOf(1)');
assert.sameValue(a.lastIndexOf(""), 99999, 'a.lastIndexOf("")');
assert.sameValue(a.lastIndexOf("str"), 123456, 'a.lastIndexOf("str")');
assert.sameValue(a.lastIndexOf(5.5), 5555, 'a.lastIndexOf(5.5)');
assert.sameValue(a.lastIndexOf(1E+309), 5, 'a.lastIndexOf(1E+309)');

assert.sameValue(a.lastIndexOf(true), -1, 'a.lastIndexOf(true)');
assert.sameValue(a.lastIndexOf(5), -1, 'a.lastIndexOf(5)');
assert.sameValue(a.lastIndexOf("str1"), -1, 'a.lastIndexOf("str1")');
assert.sameValue(a.lastIndexOf(null), -1, 'a.lastIndexOf(null)');
assert.sameValue(a.lastIndexOf(new Object()), -1, 'a.lastIndexOf(new Object())');

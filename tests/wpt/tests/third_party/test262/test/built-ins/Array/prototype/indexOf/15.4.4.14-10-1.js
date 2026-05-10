// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf returns -1 for elements not present in
    array
---*/

var a = new Array();
a[100] = 1;
a[99999] = "";
a[10] = new Object();
a[5555] = 5.5;
a[123456] = "str";
a[5] = 1E+309;

assert.sameValue(a.indexOf(1), 100, 'a.indexOf(1)');
assert.sameValue(a.indexOf(""), 99999, 'a.indexOf("")');
assert.sameValue(a.indexOf("str"), 123456, 'a.indexOf("str")');
assert.sameValue(a.indexOf(1E+309), 5, 'a.indexOf(1E+309)'); //Infinity
assert.sameValue(a.indexOf(5.5), 5555, 'a.indexOf(5.5)');

assert.sameValue(a.indexOf(true), -1, 'a.indexOf(true)');
assert.sameValue(a.indexOf(5), -1, 'a.indexOf(5)');
assert.sameValue(a.indexOf("str1"), -1, 'a.indexOf("str1")');
assert.sameValue(a.indexOf(null), -1, 'a.indexOf(null)');
assert.sameValue(a.indexOf(new Object()), -1, 'a.indexOf(new Object())');

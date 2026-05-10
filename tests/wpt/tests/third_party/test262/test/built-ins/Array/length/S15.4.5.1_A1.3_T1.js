// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array-exotic-objects-defineownproperty-p-desc
info: Set the value of property length of A to Uint32(length)
es5id: 15.4.5.1_A1.3_T1
description: length is object or primitve
---*/

var x = [];
x.length = true;
assert.sameValue(x.length, 1, 'The value of x.length is expected to be 1');

x = [0];
x.length = null;
assert.sameValue(x.length, 0, 'The value of x.length is expected to be 0');

x = [0];
x.length = new Boolean(false);
assert.sameValue(x.length, 0, 'The value of x.length is expected to be 0');

x = [];
x.length = new Number(1);
assert.sameValue(x.length, 1, 'The value of x.length is expected to be 1');

x = [];
x.length = "1";
assert.sameValue(x.length, 1, 'The value of x.length is expected to be 1');

x = [];
x.length = new String("1");
assert.sameValue(x.length, 1, 'The value of x.length is expected to be 1');

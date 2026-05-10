// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.4.5-1
description: Array instances have [[Class]] set to 'Array'
---*/

var a = [];
var s = Object.prototype.toString.call(a);

assert.sameValue(s, '[object Array]', 'The value of s is expected to be "[object Array]"');

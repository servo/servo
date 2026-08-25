// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The [[Class]] property of the newly constructed object is set to "RegExp"
es5id: 15.10.4.1_A6_T1
description: Checking [[Class]] property of the newly constructed object
---*/

var __re = new RegExp;
__re.toString = Object.prototype.toString;

assert.sameValue(__re.toString(), "[object "+"RegExp"+"]", '__re.toString() must return "[object "+"RegExp"+"]"');

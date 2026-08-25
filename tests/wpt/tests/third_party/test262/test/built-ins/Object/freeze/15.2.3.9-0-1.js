// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-0-1
description: Object.freeze must exist as a function
---*/

var f = Object.freeze;

assert.sameValue(typeof(f), "function", 'typeof(f)');

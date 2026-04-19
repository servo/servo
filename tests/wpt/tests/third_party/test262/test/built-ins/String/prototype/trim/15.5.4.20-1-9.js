// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-1-9
description: >
    String.prototype.trim works for a String object which value is
    undefined
---*/

var strObj = new String(undefined);

assert.sameValue(strObj.trim(), "undefined", 'strObj.trim()');

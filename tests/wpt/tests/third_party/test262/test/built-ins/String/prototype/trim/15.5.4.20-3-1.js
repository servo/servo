// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-3-1
description: String.prototype.trim - 'S' is a string with all LineTerminator
---*/

var lineTerminatorsStr = "\u000A\u000D\u2028\u2029";

assert.sameValue(lineTerminatorsStr.trim(), "", 'lineTerminatorsStr.trim()');

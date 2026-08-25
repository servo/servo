// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-4-59
description: >
    String.prototype.trim handles whitepace and lineterminators
    (\u2029abc as a multiline string)
---*/

var s = "\u2029\
           abc";

assert.sameValue(s.trim(), "abc", 's.trim()');

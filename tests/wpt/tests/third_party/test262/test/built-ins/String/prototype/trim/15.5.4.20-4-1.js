// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-4-1
description: >
    String.prototype.trim handles multiline string with whitepace and
    lineterminators
---*/

var s = "\u0009a b\
c \u0009"



assert.sameValue(s.trim(), "a bc", 's.trim()');

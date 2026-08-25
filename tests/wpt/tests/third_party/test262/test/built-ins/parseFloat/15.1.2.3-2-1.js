// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-parsefloat-string
description: >
    pareseFloat - 'trimmedString' is the empty string when inputString
    does not contain any such characters
---*/

assert.sameValue(parseFloat(""), NaN, 'parseFloat("")');

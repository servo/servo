// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.13.2-20-s
description: >
    ReferenceError isn't thrown if the LeftHandSideExpression of a Compound
    Assignment operator(&=) evaluates to a resolvable reference
---*/

        var _11_13_2_20 = 5
        _11_13_2_20 &= 3;

assert.sameValue(_11_13_2_20, 1, '_11_13_2_20');

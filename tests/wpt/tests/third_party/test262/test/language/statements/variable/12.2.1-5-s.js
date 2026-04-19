// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.2.1-5-s
description: >
    a Function declaring var named 'eval' does not throw SyntaxError
---*/

        Function('var eval;');

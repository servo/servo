// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-68-s
description: >
    checking 'this' (strict function declaration called by
    Function.prototype.apply(undefined))
---*/

function f() { "use strict"; return this===undefined;};

assert(f.apply(undefined), 'f.apply(undefined) !== true');

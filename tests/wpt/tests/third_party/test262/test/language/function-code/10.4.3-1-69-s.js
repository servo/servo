// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-69-s
description: >
    checking 'this' (strict function declaration called by
    Function.prototype.apply(someObject))
---*/

var o = {};
function f() { "use strict"; return this===o;};

assert(f.apply(o), 'f.apply(o) !== true');

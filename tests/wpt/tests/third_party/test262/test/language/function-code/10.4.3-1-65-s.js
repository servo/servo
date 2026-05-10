// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-65-s
description: >
    checking 'this' (strict function declaration called by non-strict new'ed
    Function constructor)
---*/

this.f = function()  { "use strict"; return this===undefined;};
assert((new Function("return f();"))());

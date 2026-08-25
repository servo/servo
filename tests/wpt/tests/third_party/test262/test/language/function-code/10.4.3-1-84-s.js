// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-84-s
description: >
    Strict Mode - checking 'this' (non-strict function declaration
    called by strict new'ed Function constructor)
flags: [noStrict]
---*/

this.f = function()  { return this!==undefined;};
assert((function () {return new Function("\"use strict\";return f();")();})());

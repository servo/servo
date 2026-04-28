// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The initial value of Error.prototype.constructor is the built-in Error
    constructor
es5id: 15.11.4.1_A1_T2
description: >
    Checking if creating "new Error.prototype.constructor" passes and
    checking its properties
---*/

var constr = Error.prototype.constructor;

var err = new constr;

assert.notSameValue(err, undefined, 'The value of err is expected to not equal ``undefined``');
assert.sameValue(err.constructor, Error, 'The value of err.constructor is expected to equal the value of Error');
assert(Error.prototype.isPrototypeOf(err), 'Error.prototype.isPrototypeOf(err) must return true');
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
// CHECK#3
Error.prototype.toString = Object.prototype.toString;
assert.sameValue(err.toString(), '[object Error]', 'err.toString() must return "[object Error]"');
assert.sameValue(err.valueOf().toString(), '[object Error]', 'err.valueOf().toString() must return "[object Error]"');

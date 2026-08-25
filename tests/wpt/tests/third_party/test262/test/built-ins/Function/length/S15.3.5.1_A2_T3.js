// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: the length property does not have the attributes { DontDelete }
es5id: 15.3.5.1_A2_T3
description: >
    Checking if deleting the length property of
    Function("arg1,arg2,arg3","arg1,arg2","arg3", null) succeeds
---*/

var f = new Function("arg1,arg2,arg3", "arg1,arg2", "arg3", null);

assert(f.hasOwnProperty('length'), 'f.hasOwnProperty(\'length\') must return true');

delete f.length;

assert(!f.hasOwnProperty('length'), 'The value of !f.hasOwnProperty(\'length\') is expected to be true');
assert.notSameValue(f.length, 6, 'The value of f.length is not 6');

// TODO: Convert to verifyProperty() format.

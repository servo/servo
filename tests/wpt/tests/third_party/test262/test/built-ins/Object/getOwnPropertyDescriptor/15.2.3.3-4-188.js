// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-188
description: >
    Object.getOwnPropertyDescriptor returns undefined for non-existent
    properties on built-ins (Function (instance).name)
---*/

var f = Function('return 42;');
var desc = Object.getOwnPropertyDescriptor(f, "functionNameHopefullyDoesNotExist");

assert.sameValue(desc, undefined, 'desc');

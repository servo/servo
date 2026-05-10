// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The [[Class]] property of the newly constructed object
    is set to "Boolean"
esid: sec-boolean-constructor
description: For testing toString function is used
---*/

delete Boolean.prototype.toString;

var obj = new Boolean();

assert.sameValue(obj.toString(), "[object Boolean]", 'obj.toString() must return "[object Boolean]"');

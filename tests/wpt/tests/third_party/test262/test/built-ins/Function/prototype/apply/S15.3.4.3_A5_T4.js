// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If thisArg is not null(defined) the called function is passed
    ToObject(thisArg) as the this value
es5id: 15.3.4.3_A5_T4
description: thisArg is function variable that return this
flags: [noStrict]
---*/

f = function() {
  this.touched = true;
  return this;
};

retobj = f.apply(obj);

assert.sameValue(typeof obj, "undefined", 'The value of `typeof obj` is expected to be "undefined"');
assert(retobj["touched"], 'The value of retobj["touched"] is expected to be true');

var obj;

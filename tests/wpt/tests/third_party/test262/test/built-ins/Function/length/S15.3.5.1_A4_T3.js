// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: the length property has the attributes { DontEnum }
es5id: 15.3.5.1_A4_T3
description: >
    Checking if enumerating the length property of
    Function("arg1,arg2,arg3","arg1,arg2","arg3", null) fails
---*/

var f = new Function("arg1,arg2,arg3", "arg1,arg2", "arg3", null);

assert(f.hasOwnProperty('length'), 'f.hasOwnProperty(\'length\') must return true');

for (var key in f) {
  if (key == "length") {
    var lengthenumed = true;
  }
}

assert(!lengthenumed, 'The value of !lengthenumed is expected to be true');

// TODO: Convert to verifyProperty() format.

// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator "void" uses GetValue
es5id: 11.4.2_A2_T2
description: If GetBase(x) is null, throw ReferenceError
---*/

assert.throws(ReferenceError, function() {
  void x;
});

// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.11-1
description: >
    'this' object used by the replaceValue function of a
    String.prototype.replace invocation
flags: [noStrict]
---*/

var global = this;
var retVal = 'x'.replace(/x/,
  function() {
    if (this === global) {
      return 'y';
    } else {
      return 'z';
    }
  });

assert.sameValue(retVal, 'y', 'retVal');

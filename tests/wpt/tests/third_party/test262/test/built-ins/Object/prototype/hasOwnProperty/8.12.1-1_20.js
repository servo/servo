// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 8.12.1-1_20
description: Properties - [[HasOwnProperty]] (literal own getter property)
---*/

var o = {
  get foo() {
    return 42;
  }
};

assert(o.hasOwnProperty("foo"), 'o.hasOwnProperty("foo") !== true');

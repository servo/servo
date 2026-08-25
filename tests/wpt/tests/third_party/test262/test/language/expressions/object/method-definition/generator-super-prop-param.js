// Copyright 2015 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
 GeneratorMethod can reference SuperProperty in default parameters
es6id: 14.4.1
author: Sam Mikes
description: GeneratorMethod uses SuperProperty (allowed)
features: [default-parameters, generators, super]
---*/

var obj = {
  *foo(a = super.toString) {
    return a;
  }
};

obj.toString = null;

assert.sameValue(obj.foo().next().value, Object.prototype.toString);

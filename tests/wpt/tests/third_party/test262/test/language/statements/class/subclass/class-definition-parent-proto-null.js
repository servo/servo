// Copyright (C) 2016 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: A class which extends a constructor with null .prototype is a derived class.
---*/

var invoked = false;
var instance, savedArg;

function A(arg) {
  invoked = true;
  savedArg = arg;
  this.prop = 0;
}
A.prototype = null;

class C extends A {}

instance = new C(1);

assert.sameValue(invoked, true);
assert.sameValue(savedArg, 1);
assert.sameValue(instance.prop, 0);

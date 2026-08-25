// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: >
  Test the first argument(target) of Object.Assign(target,...sources),
  if target is Object,its properties will be the properties of new object.
es6id:  19.1.2.1.1
---*/

var target = {
  foo: 1
};
var result = Object.assign(target, {
  a: 2
});

assert.sameValue(result.foo, 1, "The value should be {foo: 1}.");
assert.sameValue(result.a, 2, "The value should be {a: 2}.");

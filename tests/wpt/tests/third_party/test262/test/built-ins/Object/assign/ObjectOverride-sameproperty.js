// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: >
  Object properties are assigned to target in ascending index order,
  i.e. a later assignment to the same property overrides an earlier assignment.
es6id:  19.1.2.1
---*/

var target = {
  a: 1
};
var result = Object.assign(target, {
  a: 2
}, {
  a: "c"
});

assert.sameValue(result.a, "c", "The value should be 'c'.");

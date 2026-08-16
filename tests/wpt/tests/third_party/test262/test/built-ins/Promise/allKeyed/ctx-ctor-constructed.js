// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allkeyed
description: >
  Promise.allKeyed invokes the constructor via Construct (not Call), passing
  a single executor argument that is a function with length 2.
info: |
  Promise.allKeyed ( promises )

  1. Let C be the this value.
  2. Let promiseCapability be ? NewPromiseCapability(C).

  NewPromiseCapability ( C )

  ...
  6. Let promise be Construct(C, « executor »).
features: [await-dictionary, new.target]
---*/

var error = new Test262Error();

function Constructor(executor) {
  if (new.target !== Constructor) {
    throw error;
  }
  assert.sameValue(arguments.length, 1, "expected exactly one argument");
  assert.sameValue(typeof executor, "function", "executor is a function");
  assert.sameValue(executor.length, 2, "executor.length === 2");
  executor(function() {}, function() {});
}
Constructor.resolve = function(v) { return v; };

Promise.allKeyed.call(Constructor, { a: 1 });

// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createbuiltinfunction
description: Promise executor function property order
info: |
  Set order: "length", "name"
---*/

var executorFunction;

function NotPromise(executor) {
  executorFunction = executor;
  executor(function() {}, function() {});
}
Promise.resolve.call(NotPromise);

var propNames = Object.getOwnPropertyNames(executorFunction);
var lengthIndex = propNames.indexOf("length");
var nameIndex = propNames.indexOf("name");

assert(lengthIndex >= 0 && nameIndex === lengthIndex + 1,
  "The `length` property comes before the `name` property on built-in functions");

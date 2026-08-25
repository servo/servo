// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createbuiltinfunction
description: "%ThrowTypeError% function property order"
info: |
  Set order: "length", "name"
---*/

var ThrowTypeError = Object.getOwnPropertyDescriptor(function() {
  "use strict";
  return arguments;
}(), "callee").get;

var propNames = Object.getOwnPropertyNames(ThrowTypeError);
var lengthIndex = propNames.indexOf("length");
var nameIndex = propNames.indexOf("name");

assert(lengthIndex >= 0 && nameIndex === lengthIndex + 1,
  "The `length` property comes before the `name` property on built-in functions");

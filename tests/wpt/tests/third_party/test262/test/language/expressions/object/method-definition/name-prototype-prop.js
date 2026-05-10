// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Functions declared as methods do not define a `prototype` property.
es6id: 14.3.9
---*/

var method = { method() {} }.method;

assert(
  !Object.prototype.hasOwnProperty.call(method, 'prototype'),
  "Functions declared as methods do not define a 'prototype' property"
);

// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    ClassExpression
---*/
var A = class {}

assert.sameValue(typeof A, "function");

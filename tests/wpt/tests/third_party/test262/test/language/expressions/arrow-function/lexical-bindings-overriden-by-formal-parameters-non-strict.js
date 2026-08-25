// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.2.16
description: >
    Runtime Semantics: Evaluation

flags: [noStrict]
---*/
function f() {
  return (arguments) => arguments;
}

assert.sameValue(f(1)(2), 2);

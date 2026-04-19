// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.2.16
description: >
    Runtime Semantics: Evaluation

    1. If the function code for this ArrowFunction is strict mode code (10.2.1),
      let strict be true. Otherwise let strict be false.
    ...

flags: [noStrict]
---*/
var af = _ => {
  foo = 1;
};

af();

assert.sameValue(foo, 1);

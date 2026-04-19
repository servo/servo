// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-web-compat-evaldeclarationinstantiation
description: >
    AnnexB extension not honored in strict mode, Function declaration
    in the `default` clause of a `switch` statement in eval code
info: |
    B.3.3.3 Changes to EvalDeclarationInstantiation

    1. If strict is false, then
      ...

flags: [noStrict]
---*/

var err;

eval('\
  "use strict";\
  switch (1) {\
    default:\
      function f() {  }\
  }\
');

try {
  f;
} catch (exception) {
  err = exception;
}

assert.sameValue(err.constructor, ReferenceError);

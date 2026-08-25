// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: AnnexB extension not honored in strict mode (Function declaration in the `case` clause of a `switch` statement in the global scope)
es6id: B.3.3.2
flags: [onlyStrict]
info: |
    B.3.3.2 Changes to GlobalDeclarationInstantiation

    1. 1. Let strict be IsStrict of script
    2. If strict is *false*, then
       [...]
---*/

assert.throws(ReferenceError, function() {
  f;
});

switch (1) {
  case 1:
    function f() {  }
}

assert.throws(ReferenceError, function() {
  f;
});

// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: AnnexB extension not honored in strict mode (Block statement in the global scope containing a function declaration)
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

{
  function f() {  }
}

assert.throws(ReferenceError, function() {
  f;
});

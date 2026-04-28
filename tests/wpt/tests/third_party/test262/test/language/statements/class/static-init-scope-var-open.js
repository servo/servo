// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classstaticblockdefinitionevaluation
description: Creation of new environment record for variable-scoped bindings
info: |
  ClassStaticBlock : static { ClassStaticBlockBody }

    1. Let lex be the running execution context's LexicalEnvironment.
    2. Let privateScope be the running execution context's PrivateEnvironment.
    3. Let body be OrdinaryFunctionCreate(Method, « », ClassStaticBlockBody, lex, privateScope).
features: [class-static-block]
---*/

var test262 = 'outer scope';
var probe1, probe2;

class C {
  static {
    var test262 = 'first block';
    probe1 = test262;
  }
  static {
    var test262 = 'second block';
    probe2 = test262;
  }
}

assert.sameValue(test262, 'outer scope');
assert.sameValue(probe1, 'first block');
assert.sameValue(probe2, 'second block');

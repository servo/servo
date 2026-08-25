// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classstaticblockdefinitionevaluation
description: Shares environment record for privately-scoped bindings
info: |
  ClassStaticBlock : static { ClassStaticBlockBody }

    1. Let lex be the running execution context's LexicalEnvironment.
    2. Let privateScope be the running execution context's PrivateEnvironment.
    3. Let body be OrdinaryFunctionCreate(Method, « », ClassStaticBlockBody, lex, privateScope).
features: [class-fields-private, class-static-block]
---*/

var probe;

class C {
  static #test262 = 'private';

  static {
    probe = C.#test262;
  }
}

assert.sameValue(probe, 'private');

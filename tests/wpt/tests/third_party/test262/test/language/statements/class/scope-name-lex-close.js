// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: Removal of lexical environment for class "name"
info: |
    [...]
    22. Set the running execution context's LexicalEnvironment to lex.
    [...]
---*/

class C {
  method() {
    return C;
  }
}

var cls = C;
assert.sameValue(typeof C, 'function');
C = null;
assert.sameValue(C, null);
assert.sameValue(cls.prototype.method(), cls, 'from instance method');

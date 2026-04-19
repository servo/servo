// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: >
    Creation of new lexical environment for the class "name" (without a
    heritage)
info: |
    1. Let lex be the LexicalEnvironment of the running execution context.
    2. Let classScope be NewDeclarativeEnvironment(lex).
    3. Let classScopeEnvRec be classScope's EnvironmentRecord.
    4. If className is not undefined, then
       a. Perform classScopeEnvRec.CreateImmutableBinding(className, true).
    5. If ClassHeritageopt is not present, then
       [...]
    6. Else,
       a. Set the running execution context's LexicalEnvironment to classScope.
       [...]
    11. Set the running execution context's LexicalEnvironment to classScope.
---*/

var probeBefore = function() { return C; };
var C = 'outside';

var cls = class C {
  probe() {
    return C;
  }
  modify() {
    C = null;
  }
};

assert.sameValue(probeBefore(), 'outside');
assert.sameValue(cls.prototype.probe(), cls, 'inner binding value');
assert.throws(
  TypeError, cls.prototype.modify, 'inner binding rejects modification'
);
assert.sameValue(cls.prototype.probe(), cls, 'inner binding is immutable');

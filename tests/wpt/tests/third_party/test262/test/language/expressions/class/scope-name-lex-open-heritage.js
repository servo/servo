// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: >
    Creation of new lexical environment for the class "name" (with a heritage)
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
---*/

var probeBefore = function() { return C; };
var probeHeritage, setHeritage;
var C = 'outside';

var cls = class C extends (
    probeHeritage = function() { return C; },
    setHeritage = function() { C = null; }
  ) {
  method() {
    return C;
  }
};

assert.sameValue(probeBefore(), 'outside');
assert.sameValue(probeHeritage(), cls, 'from class heritage');
assert.throws(TypeError, setHeritage, 'inner binding rejects modification');
assert.sameValue(probeHeritage(), cls, 'inner binding is immutable');
assert.sameValue(cls.prototype.method(), cls, 'from instance method');

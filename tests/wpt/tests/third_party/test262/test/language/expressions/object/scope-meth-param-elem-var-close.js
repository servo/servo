// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-functiondeclarationinstantiation
description: >
    sloppy direct evals in params introduce vars
info: |
    [...]
    20. Else,
      a. NOTE: A separate Environment Record is needed to ensure that bindings created by direct eval calls in the formal parameter list are outside the environment where parameters are declared.
      b. Let calleeEnv be the LexicalEnvironment of calleeContext.
      c. Let env be NewDeclarativeEnvironment(calleeEnv).
      d. Let envRec be env's EnvironmentRecord.
    [...]
flags: [noStrict]
---*/

var x = 'outside';
var probe1, probe2, probeBody;

({
  m(
    _ = (eval('var x = "inside";'), probe1 = function() { return x; }),
    __ = probe2 = function() { return x; }
  ) {
    probeBody = function() { return x; };
  }
}.m());

assert.sameValue(probe1(), 'inside');
assert.sameValue(probe2(), 'inside');
assert.sameValue(probeBody(), 'inside');

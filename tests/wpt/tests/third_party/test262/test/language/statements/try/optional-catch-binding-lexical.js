// Copyright (C) 2017 Lucas Azzola. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Lucas Azzola
esid: pending
description: lexical environment runtime semantics for optional catch binding
features: [optional-catch-binding]
info: |
  Runtime Semantics: CatchClauseEvaluation

  Catch : catch Block
    Let oldEnv be the running execution context's LexicalEnvironment.
    Let catchEnv be NewDeclarativeEnvironment(oldEnv).
    Set the running execution context's LexicalEnvironment to catchEnv.
    (...)
    Set the running execution context's LexicalEnvironment to oldEnv.
    Return Completion(B).
---*/

let x = 1;
let ranCatch = false;

try {
    x = 2;
    throw new Error();
} catch {
    let x = 3;
    let y = true;
    ranCatch = true;
}

assert(ranCatch, 'executed `catch` block');
assert.sameValue(x, 2);

assert.throws(ReferenceError, function() {
    y;
});

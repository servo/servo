// Copyright (C) 2017 Lucas Azzola. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Lucas Azzola
esid: pending
description: errors can be thrown from catch clause without binding
features: [optional-catch-binding]
info: |
  Runtime Semantics: CatchClauseEvaluation

  Catch : catch Block
    (...)
    Let B be the result of evaluating Block.
    (...)
    Return Completion(B).
---*/

assert.throws(Test262Error, function() {
    try {
        throw new Error();
    } catch {
        throw new Test262Error();
    }
});

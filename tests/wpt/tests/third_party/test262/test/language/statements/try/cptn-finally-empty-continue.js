// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-try-statement-runtime-semantics-evaluation
description: Abrupt completion from finally block calls UpdatEmpty()
info: |
  13.15.8 Runtime Semantics: Evaluation
  TryStatement : try Block Finally
    ...
    2. Let F be the result of evaluating Finally.
    ...
    4. Return Completion(UpdateEmpty(F, undefined)).
---*/

// Ensure the completion value from the first iteration ('bad completion') is not returned.
var completion = eval("for (var i = 0; i < 2; ++i) { if (i) { try {} finally { continue; } } 'bad completion'; }");
assert.sameValue(completion, undefined);

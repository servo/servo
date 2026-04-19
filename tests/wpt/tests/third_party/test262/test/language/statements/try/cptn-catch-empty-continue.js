// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-try-statement-runtime-semantics-evaluation
description: Abrupt completion from catch block calls UpdatEmpty()
info: |
  13.15.8 Runtime Semantics: Evaluation
  TryStatement : try Block Catch
    ...
    2. If B.[[Type]] is throw, let C be CatchClauseEvaluation of Catch with parameter B.[[Value]].
    ...
    4. Return Completion(UpdateEmpty(C, undefined)).
---*/

// Ensure the completion value from the first iteration ('bad completion') is not returned.
var completion = eval("for (var i = 0; i < 2; ++i) { if (i) { try { throw null; } catch (e) { continue; } } 'bad completion'; }");
assert.sameValue(completion, undefined);

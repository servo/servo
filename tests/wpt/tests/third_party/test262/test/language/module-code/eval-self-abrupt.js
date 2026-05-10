// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Abrupt completion from module evaluation is reported
esid: sec-moduleevaluation
info: |
    [...]
    16. Let result be the result of evaluating module.[[ECMAScriptCode]].
    17. Suspend moduleCxt and remove it from the execution context stack.
    18. Resume the context that is now on the top of the execution context
        stack as the running execution context.
    19. Return Completion(result).
negative:
  phase: runtime
  type: Test262Error
flags: [module]
---*/

throw new Test262Error();

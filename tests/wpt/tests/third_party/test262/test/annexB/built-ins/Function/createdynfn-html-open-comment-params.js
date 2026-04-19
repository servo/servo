// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdynamicfunction
description: >
  Create a Function with the function parameters being a html open comment.
info: |
  19.2.1.1.1 Runtime Semantics: CreateDynamicFunction(constructor, newTarget, kind, args)
    ...
    7. If kind is "normal", then
      ...
      b. Let parameterGoal be the grammar symbol FormalParameters[~Yield, ~Await].
    ...
    10. Let parameters be the result of parsing P, interpreted as UTF-16 encoded Unicode text
        as described in 6.1.4, using parameterGoal as the goal symbol. Throw a SyntaxError
        exception if the parse fails.
    ...
---*/

Function("<!--", "");

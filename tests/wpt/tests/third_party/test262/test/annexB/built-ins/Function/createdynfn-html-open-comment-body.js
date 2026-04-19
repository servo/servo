// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdynamicfunction
description: >
  Create a Function with the function body being a html open comment.
info: |
  19.2.1.1.1 Runtime Semantics: CreateDynamicFunction(constructor, newTarget, kind, args)
    ...
    7. If kind is "normal", then
      a. Let goal be the grammar symbol FunctionBody[~Yield, ~Await].
    ...
    11. Let body be the result of parsing bodyText, interpreted as UTF-16 encoded Unicode text
        as described in 6.1.4, using goal as the goal symbol. Throw a SyntaxError exception if
        the parse fails.
    ...
---*/

Function("<!--");

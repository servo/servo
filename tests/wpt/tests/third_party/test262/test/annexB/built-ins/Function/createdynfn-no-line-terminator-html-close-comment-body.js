// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdynamicfunction
description: >
  Function body is wrapped with new lines before being parsed
info: |
  The HTMLCloseComment requires a preceding line terminator.

  Runtime Semantics: CreateDynamicFunction(constructor, newTarget, kind, args)
  ...
  Set bodyText to ? ToString(bodyText).
  Let parameters be the result of parsing P, interpreted as UTF-16 encoded Unicode text as
  described in 6.1.4, using parameterGoal as the goal symbol. Throw a SyntaxError exception if the
  parse fails.
  Let body be the result of parsing bodyText, interpreted as UTF-16 encoded Unicode text as
  described in 6.1.4, using goal as the goal symbol. Throw a SyntaxError exception if the parse
  fails.
---*/

Function("-->");

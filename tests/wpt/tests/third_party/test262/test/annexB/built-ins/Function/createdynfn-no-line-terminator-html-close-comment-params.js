// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdynamicfunction
description: >
  Function parses the parameters text before forming the sourceText with the proper line feed.
info: |
  The HTMLCloseComment requires a preceding line terminator.

  Runtime Semantics: CreateDynamicFunction(constructor, newTarget, kind, args)
  ...
  16. Set bodyText to ? ToString(bodyText).
  17. Let parameters be the result of parsing P, interpreted as UTF-16 encoded Unicode text as
  described in 6.1.4, using parameterGoal as the goal symbol. Throw a SyntaxError exception if the
  parse fails.
  18. Let body be the result of parsing bodyText, interpreted as UTF-16 encoded Unicode text as
  described in 6.1.4, using goal as the goal symbol. Throw a SyntaxError exception if the parse
  fails.
  ...
  41. Let sourceText be the string-concatenation of prefix, " anonymous(", P, 0x000A (LINE FEED),
  ") {", 0x000A (LINE FEED), bodyText, 0x000A (LINE FEED), and "}".
---*/

assert.throws(SyntaxError, () => Function("-->", ""));

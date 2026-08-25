// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-template-literal-lexical-components
description: Invalid unicode escape sequence
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

`\u0`;

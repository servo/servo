// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.8
description: >
    The expression within the template should be evaluated according to the
    semantics of the surrounding context.
    The SV of EscapeSequence :: HexEscapeSequence is the SV of the
    HexEscapeSequence.
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

`${'\07'}`;

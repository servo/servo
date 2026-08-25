// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-left-hand-side-expressions
description: >
  "import" must not contain escape sequences.
info: |
  5.1.5 Grammar Notation

  Terminal symbols are shown in fixed width
  font, both in the productions of the grammars and throughout this specification whenever the
  text directly refers to such a terminal symbol. These are to appear in a script exactly as
  written. All terminal symbol code points specified in this way are to be understood as the
  appropriate Unicode code points from the Basic Latin range, as opposed to any similar-looking
  code points from other Unicode ranges.

  CallExpression :
    MemberExpressionArguments
    SuperCall
    ImportCall
    CallExpressionArguments
    CallExpressionTemplateLiteral

  ImportCall :
    import( AssignmentExpression )
negative:
  phase: parse
  type: SyntaxError
features: [dynamic-import]
---*/

$DONOTEVALUATE();

im\u0070ort('./empty_FIXTURE.js');

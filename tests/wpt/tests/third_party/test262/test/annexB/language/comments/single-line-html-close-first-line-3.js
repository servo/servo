/* a comment */ /*another comment*/--> a comment

// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-html-like-comments
description: >
    A SingleLineHTMLCloseComment is allowed in the first line when preceeded by spaces and single-line block comments
flags: [raw]
info: |
    InputElementHashbangOrRegExp ::
      WhiteSpace
      LineTerminator
      Comment
      CommonToken
      HashbangComment
      RegularExpressionLiteral
      HTMLCloseComment

    HTMLCloseComment ::
      WhiteSpaceSequence[opt] SingleLineDelimitedCommentSequence[opt] --> SingleLineCommentChars[opt]
negative:
  phase: runtime
  type: EvalError
---*/

// Because this test concerns the interpretation of non-executable character
// sequences within ECMAScript source code, special care must be taken to
// ensure that executable code is evaluated as expected.
//
// Express the intended behavior by intentionally throwing an error; this
// guarantees that test runners will only consider the test "passing" if
// executable sequences are correctly interpreted as such.
throw new EvalError("This is not in a comment");

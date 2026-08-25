// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-html-like-comments
description: An HTMLCloseComment must be preceded by a LineTerminator
info: |
    Comment ::
      MultiLineComment
      SingleLineComment
      SingleLineHTMLOpenComment
      SingleLineHTMLCloseComment
      SingleLineDelimitedComment

    HTMLCloseComment ::
      WhiteSpaceSequence[opt] SingleLineDelimitedCommentSequence[opt] --> SingleLineCommentChars[opt]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

;-->

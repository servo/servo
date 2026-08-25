// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-html-like-comments
description: >
    Arbitrary character sequence not permitted before HTMLCloseComment token
info: |
    Comment ::
      MultiLineComment
      SingleLineComment
      SingleLineHTMLOpenComment
      SingleLineHTMLCloseComment
      SingleLineDelimitedComment

    MultiLineComment ::
      /* FirstCommentLine[opt] LineTerminator MultiLineCommentChars[opt] * / HTMLCloseComment[opt]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

/*
*/ the comment should not include these characters, regardless of AnnexB extensions -->

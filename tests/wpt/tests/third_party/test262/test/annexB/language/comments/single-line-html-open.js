// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-html-like-comments
description: SingleLineHTMLOpenComment
info: |
    Comment ::
      MultiLineComment
      SingleLineComment
      SingleLineHTMLOpenComment
      SingleLineHTMLCloseComment
      SingleLineDelimitedComment

    SingleLineHTMLOpenComment ::
      <!--SingleLineCommentCharsopt
negative:
  phase: runtime
  type: Test262Error
---*/

var counter = 0;
<!--
counter += 1;

<!--the comment extends to these characters
counter += 1;

counter += 1;<!--the comment extends to these characters
counter += 1;

var x = 0;
x = -1 <!--x;

// Because this test concerns the interpretation of non-executable character
// sequences within ECMAScript source code, special care must be taken to
// ensure that executable code is evaluated as expected.
//
// Express the intended behavior by intentionally throwing an error; this
// guarantees that test runners will only consider the test "passing" if
// executable sequences are correctly interpreted as such.
if (counter === 4 && x === -1) {
  throw new Test262Error();
}

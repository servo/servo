// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-html-like-comments
description: SingleLineHTMLCloseComment
info: |
    Comment ::
      MultiLineComment
      SingleLineComment
      SingleLineHTMLOpenComment
      SingleLineHTMLCloseComment
      SingleLineDelimitedComment

    SingleLineHTMLCloseComment ::
      LineTerminatorSequenceHTMLCloseComment

    HTMLCloseComment ::
      WhiteSpaceSequence[opt] SingleLineDelimitedCommentSequence[opt] --> SingleLineCommentChars[opt]
negative:
  phase: runtime
  type: Test262Error
---*/

var counter = 0;

// DANGER WILL ROBINSON!
//
// There are UTF-8-encoded Unicode separators in the lines below.  Some text
// editors (notably including, in the experience of this test's author, the
// GNOME Text Editor used to attempt to create this file) don't properly insert
// and save both these characters.  (It seemed to handle copy/pasting U+2028
// LINE SEPARATOR from GNOME Character Map just fine, but U+2029 PARAGRAPH
// SEPARATOR got mangled in the final saved file.)  Be extremely careful editing
// this file to not inadvertently break this test.

counter -->a U+2028 LINE SEPARATOR between "counter" and "-->" means this is all a comment
counter += 1;

counter -->a U+2029 PARAGRAPH SEPARATOR between "counter" and "-->" means this is all a comment
counter += 1;

// Because this test concerns the interpretation of non-executable character
// sequences within ECMAScript source code, special care must be taken to
// ensure that executable code is evaluated as expected.
//
// Express the intended behavior by intentionally throwing an error; this
// guarantees that test runners will only consider the test "passing" if
// executable sequences are correctly interpreted as such.
if (counter === 2) {
  throw new Test262Error();
}

// Copyright (C) 2023 Veera Sivarajan. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-keywords-and-reserved-words
description: >
  ReservedWord (false) cannot contain UnicodeEscapeSequence.
info: |
  Note 1

  Per 5.1.5, keywords in the grammar match literal sequences of specific SourceCharacter elements.
  A code point in a keyword cannot be expressed by a \ UnicodeEscapeSequence.
negative: 
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

f\u{61}lse;

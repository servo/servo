// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Punctuator cannot be expressed as a Unicode escape sequence consisting of
    six characters, namely \u plus four hexadecimal digits
es5id: 7.7_A2_T2
description: Try to use () as Unicode \u00281\u0029
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

\u00281\u0029;

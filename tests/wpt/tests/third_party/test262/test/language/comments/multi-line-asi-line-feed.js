// Copyright (c) 2018 Mike Pennisi.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 7.4
esid: sec-comments
description: >
  A multi-line comment containing a line feed should be considered a
  LineTerminator
info: >
  Comments behave like white space and are discarded except that, if a
  MultiLineComment contains a line terminator code point, then the entire
  comment is considered to be a LineTerminator for purposes of parsing by the
  syntactic grammar.
---*/

''/*
*/''

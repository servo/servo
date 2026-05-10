// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function.prototype.tostring
description: Function.prototype.toString on a function with Unicode escape sequences
info: |
  Function.prototype.toString returns a slice of the source text before
  any potential Unicode escape sequence substitution in identifiers
includes: [nativeFunctionMatcher.js]
---*/

function \u0061(\u{62}, \u0063) { \u0062 = \u{00063}; return b; }

assertToStringOrNativeFunction(a, "function \\u0061(\\u{62}, \\u0063) { \\u0062 = \\u{00063}; return b; }");

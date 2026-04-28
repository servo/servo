// Copyright (C) 2016 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-parsefloat-string
description: >
  Leading U+180E is not recognized as whitespace
info: |
  18.2.4 parseFloat (string)

  ...
  3. Let trimmedString be a substring of inputString consisting of the
     leftmost code unit that is not a StrWhiteSpaceChar and all code units
     to the right of that code unit. (In other words, remove leading white
     space.) If inputString does not contain any such code units, let
     trimmedString be the empty string.
  4. If neither trimmedString nor any prefix of trimmedString satisfies the
     syntax of a StrDecimalLiteral (see 7.1.3.1), return NaN.
  ...
features: [u180e]
---*/

var mongolianVowelSeparator = "\u180E";

assert.sameValue(parseFloat(mongolianVowelSeparator + "1.1"), NaN, "Single leading U+180E");
assert.sameValue(parseFloat(mongolianVowelSeparator + mongolianVowelSeparator + mongolianVowelSeparator + "1.1"), NaN, "Multiple leading U+180E");
assert.sameValue(parseFloat(mongolianVowelSeparator), NaN, "Only U+180E");

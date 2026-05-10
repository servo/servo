// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If neither x, nor y is a prefix of each other, returned result of strings
    comparison applies a simple lexicographic ordering to the sequences of
    code unit value values
es5id: 11.8.2_A4.12_T1
description: x and y are string primitives
---*/

//CHECK#1
if (("xy" > "xx") !== true) {
  throw new Test262Error('#1: ("xy" > "xx") === true');
}

//CHECK#2
if (("xx" > "xy") !== false) {
  throw new Test262Error('#2: ("xx" > "xy") === false');
}

//CHECK#3
if (("y" > "x") !== true) {
  throw new Test262Error('#3: ("y" > "x") === true');
}

//CHECK#4
if (("aba" > "aab") !== true) {
  throw new Test262Error('#4: ("aba" > aab") === true');
}

//CHECK#5
if (("\u0061\u0061\u0061\u0061" > "\u0061\u0061\u0061\u0062") !== false) {
  throw new Test262Error('#5: ("\\u0061\\u0061\\u0061\\u0061" > \\u0061\\u0061\\u0061\\u0062") === false');
}

//CHECK#6
if (("a\u0000b" > "a\u0000a") !== true) {
  throw new Test262Error('#6: ("a\\u0000b" > "a\\u0000a") === true');
}

//CHECK#7
if (("aa" > "aB") !== true) {
  throw new Test262Error('#7: ("aa" > aB") === true');
}

//CHECK#8
if (("\u{10000}" > "\uD7FF") !== true) {
  throw new Test262Error('#8: ("\\u{10000}" > "\\uD7FF") === true');
}

//CHECK#9
if (("\uDC00" > "\uD800") !== true) {
  throw new Test262Error('#9: ("\\uDC00" > "\\uD800") === true');
}

//CHECK#10
// String comparison is done by code units, rather than by code points.
// "\u{10000}" is equivalent to "\uD800\uDC00"
if (("\u{10000}" > "\uFFFF") !== false) {
  throw new Test262Error('#10: ("\\u{10000}" > "\\uFFFF") === false');
}

//CHECK#11
if (("\u{12345}" > "\u{10000}") !== true) {
  throw new Test262Error('#11: ("\\u{12345}" > "\\u{10000}") === true');
}

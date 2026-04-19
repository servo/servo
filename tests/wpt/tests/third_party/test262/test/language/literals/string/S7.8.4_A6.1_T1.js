// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "EscapeSequence :: HexEscapeSequence :: x HexDigit HexDigit"
es5id: 7.8.4_A6.1_T1
description: "HexEscapeSequence ::  HexDigit"
---*/

//CHECK#0
if ("\x00" !== String.fromCharCode("0")) {
  throw new Test262Error('#0: "\\x00" === String.fromCharCode("0")');
}

//CHECK#1
if ("\x01" !== String.fromCharCode("1")) {
  throw new Test262Error('#1: "\\x01" === String.fromCharCode("1")');
}

//CHECK#2
if ("\x02" !== String.fromCharCode("2")) {
  throw new Test262Error('#2: "\\x02" === String.fromCharCode("2")');
}

//CHECK#3
if ("\x03" !== String.fromCharCode("3")) {
  throw new Test262Error('#3: "\\x03" === String.fromCharCode("3")');
}

//CHECK#4
if ("\x04" !== String.fromCharCode("4")) {
  throw new Test262Error('#4: "\\x04" === String.fromCharCode("4")');
}

//CHECK#5
if ("\x05" !== String.fromCharCode("5")) {
  throw new Test262Error('#5: "\\x05" === String.fromCharCode("5")');
}

//CHECK#6
if ("\x06" !== String.fromCharCode("6")) {
  throw new Test262Error('#6: "\\x06" === String.fromCharCode("6")');
}

//CHECK#7
if ("\x07" !== String.fromCharCode("7")) {
  throw new Test262Error('#7: "\\x07" === String.fromCharCode("7")');
}

//CHECK#8
if ("\x08" !== String.fromCharCode("8")) {
  throw new Test262Error('#8: "\\x08" === String.fromCharCode("8")');
}

//CHECK#9
if ("\x09" !== String.fromCharCode("9")) {
  throw new Test262Error('#9: "\\x09" === String.fromCharCode("9")');
}

//CHECK#A
if ("\x0A" !== String.fromCharCode("10")) {
  throw new Test262Error('#A: "\\x0A" === String.fromCharCode("10")');
}

//CHECK#B
if ("\x0B" !== String.fromCharCode("11")) {
  throw new Test262Error('#B: "\\x0B" === String.fromCharCode("11")');
}

//CHECK#C
if ("\x0C" !== String.fromCharCode("12")) {
  throw new Test262Error('#C: "\\x0C" === String.fromCharCode("12")');
}

//CHECK#D
if ("\x0D" !== String.fromCharCode("13")) {
  throw new Test262Error('#D: "\\x0D" === String.fromCharCode("13")');
}

//CHECK#E
if ("\x0E" !== String.fromCharCode("14")) {
  throw new Test262Error('#E: "\\x0E" === String.fromCharCode("14")');
}

//CHECK#F
if ("\x0F" !== String.fromCharCode("15")) {
  throw new Test262Error('#F: "\\x0F" === String.fromCharCode("15")');
}

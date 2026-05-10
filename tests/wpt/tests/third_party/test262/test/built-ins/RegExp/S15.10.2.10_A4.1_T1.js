// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    CharacterEscape :: UnicodeEscapeSequence :: u HexDigit HexDigit HexDigit
    HexDigit
es5id: 15.10.2.10_A4.1_T1
description: RegExp and tested string include uncode symbols
---*/

var arr = /\u0000/.exec("\u0000"); 
if ((arr === null) || (arr[0] !== "\u0000")) {
  throw new Test262Error('#0: var arr = /\\u0000/.exec(\\u0000); arr[0] === "\\u0000". Actual. ' + (arr && arr[0]));
}

var arr = /\u0001/.exec("\u0001"); 
if ((arr === null) || (arr[0] !== "\u0001")) {
  throw new Test262Error('#1: var arr = /\\u0001/.exec(\\u0001); arr[0] === "\\u0001". Actual. ' + (arr && arr[0]));
}

var arr = /\u000A/.exec("\u000A"); 
if ((arr === null) || (arr[0] !== "\u000A")) {
  throw new Test262Error('#2: var arr = /\\u000A/.exec(\\u000A); arr[0] === "\\u000A". Actual. ' + (arr && arr[0]));
}

var arr = /\u00FF/.exec("\u00FF"); 
if ((arr === null) || (arr[0] !== "\u00FF")) {
  throw new Test262Error('#3: var arr = /\\u00FF/.exec(\\u00FF); arr[0] === "\\u00FF". Actual. ' + (arr && arr[0]));
}

var arr = /\u0FFF/.exec("\u0FFF"); 
if ((arr === null) || (arr[0] !== "\u0FFF")) {
  throw new Test262Error('#4: var arr = /\\u0FFF/.exec(\\u0FFF); arr[0] === "\\u0FFF". Actual. ' + (arr && arr[0]));
}

var arr = /\uFFFF/.exec("\uFFFF"); 
if ((arr === null) || (arr[0] !== "\uFFFF")) {
  throw new Test262Error('#5: var arr = /\\uFFFF/.exec(\\uFFFF); arr[0] === "\\uFFFF". Actual. ' + (arr && arr[0]));
}

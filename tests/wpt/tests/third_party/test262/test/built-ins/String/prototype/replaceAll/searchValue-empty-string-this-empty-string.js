// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  Matching empty string for the this value and the searchValue
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  ...
  5. Let functionalReplace be IsCallable(replaceValue).
  6. If functionalReplace is false, then
    a. Let replaceValue be ? ToString(replaceValue). 
  ...
  10. Let position be ! StringIndexOf(string, searchString, 0).
  11. Repeat, while position is not -1
    a. Append position to the end of matchPositions.
    b. Let position be ! StringIndexOf(string, searchString, position + advanceBy).
  ...
  14. For each position in matchPositions, do
    a. If functionalReplace is true, then
      ...
    b. Else,
      ...
      ii. Let captures be a new empty List.
      iii. Let replacement be GetSubstitution(searchString, string, position, captures, undefined, replaceValue).

  StringIndexOf ( string, searchValue, fromIndex )

  ...
  4. Let len be the length of string.
  5. If searchValue is the empty string, and fromIndex <= len, return fromIndex.
  6. Let searchLen be the length of searchValue.
  7. If there exists any integer k such that fromIndex ≤ k ≤ len - searchLen and for all nonnegative integers j less than searchLen, the code unit at index k + j within string is the same as the code unit at index j within searchValue, let pos be the smallest (closest to -∞) such integer. Otherwise, let pos be -1.
  8. Return pos. 
features: [String.prototype.replaceAll]
---*/

var result = ''.replaceAll('', 'abc');
assert.sameValue(result, 'abc');

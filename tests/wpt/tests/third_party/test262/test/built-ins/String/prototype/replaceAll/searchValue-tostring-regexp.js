// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  ToString(searchValue)
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  ...
  4. Let searchString be ? ToString(searchValue).
  5. Let functionalReplace be IsCallable(replaceValue).
  6. If functionalReplace is false, then
    a. Let replaceValue be ? ToString(replaceValue). 
  ...
  14. For each position in matchPositions, do
    a. If functionalReplace is true, then
      ...
    b. Else,
      ...
      ii. Let captures be a new empty List.
      iii. Let replacement be GetSubstitution(searchString, string, position, captures, undefined, replaceValue).
features: [String.prototype.replaceAll, Symbol.replace]
---*/

var result;
var searchValue;

searchValue = /./g;

Object.defineProperty(searchValue, Symbol.replace, { value: undefined });

result = 'aa /./g /./g aa'.replaceAll(searchValue, 'z');
assert.sameValue(result, 'aa z z aa', '/./g');

searchValue = /./gy;

Object.defineProperty(searchValue, Symbol.replace, { value: undefined });

result = 'aa /./gy /./gy aa'.replaceAll(searchValue, 'z');
assert.sameValue(result, 'aa z z aa', '/./gy');

searchValue = /./gi;

Object.defineProperty(searchValue, Symbol.replace, { value: undefined });

result = 'aa /./gi /./gi aa'.replaceAll(searchValue, 'z');
assert.sameValue(result, 'aa z z aa', '/./gi');

searchValue = /./iyg;

Object.defineProperty(searchValue, Symbol.replace, { value: undefined });

result = 'aa /./giy /./iyg /./gyi /./giy aa'.replaceAll(searchValue, 'z');
assert.sameValue(result, 'aa z /./iyg /./gyi z aa', '/./iyg');

// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  replaceValue is used to replace matching positions in string
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  ...
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
features: [String.prototype.replaceAll]
---*/

var result = 'aaab a a aac'.replaceAll('aa', 'z');
assert.sameValue(result, 'zab a a zc');

result = 'aaab a a aac'.replaceAll('aa', 'a');
assert.sameValue(result, 'aab a a ac');

result = 'aaab a a aac'.replaceAll('a', 'a');
assert.sameValue(result, 'aaab a a aac');

result = 'aaab a a aac'.replaceAll('a', 'z');
assert.sameValue(result, 'zzzb z z zzc');

// Copyright (C) 2023 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.replace
description: >
  All $n and $nn substrings of the replacement template must be replaced with
  contents of the corresponding regular expression match capture group (if the
  n or nn identifies a valid capture index).
info: |
  String.prototype.replace ( searchValue, replaceValue )

  1. Let O be ? RequireObjectCoercible(this value).
  2. If searchValue is neither undefined nor null, then
    a. Let replacer be ? GetMethod(searchValue, @@replace).
    b. If replacer is not undefined, then
      i. Return ? Call(replacer, searchValue, « O, replaceValue »).

  RegExp.prototype [ @@replace ] ( string, replaceValue )

  15. For each element result of results, do
    a. Let resultLength be ? LengthOfArrayLike(result).
    b. Let nCaptures be max(resultLength - 1, 0).
    c. Let matched be ? ToString(? Get(result, "0")).
    d. Let matchLength be the length of matched.
    e. Let position be ? ToIntegerOrInfinity(? Get(result, "index")).
    f. Set position to the result of clamping position between 0 and lengthS.
    g. Let captures be a new empty List.
    h. Let n be 1.
    i. Repeat, while n ≤ nCaptures,
      ...
      iii. Append capN to captures.
      iv. NOTE: When n = 1, the preceding step puts the first element into captures (at index 0). More generally, the nth capture (the characters captured by the nth set of capturing parentheses) is at captures[n - 1].
      v. Set n to n + 1.
    j. Let namedCaptures be ? Get(result, "groups").
    k. If functionalReplace is true, then
      ...
    l. Else,
      i. If namedCaptures is not undefined, then
        1. Set namedCaptures to ? ToObject(namedCaptures).
      ii. Let replacement be ? GetSubstitution(matched, S, position, captures, namedCaptures, replaceValue).
    ...
  16. If nextSourcePosition ≥ lengthS, return accumulatedResult.
  17. Return the string-concatenation of accumulatedResult and the substring of S from nextSourcePosition.

  GetSubstitution ( matched, str, position, captures, namedCaptures, replacementTemplate )

  1. Let stringLength be the length of str.
  2. Assert: position ≤ stringLength.
  3. Let result be the empty String.
  4. Let templateRemainder be replacementTemplate.
  5. Repeat, while templateRemainder is not the empty String,
    a. NOTE: The following steps isolate ref (a prefix of templateRemainder), determine refReplacement (its replacement), and then append that replacement to result.
    ...
    f. Else if templateRemainder starts with "$" followed by 1 or more decimal digits, then
      i. If templateRemainder starts with "$" followed by 2 or more decimal digits, let digitCount be 2. Otherwise, let digitCount be 1.
      ii. Let digits be the substring of templateRemainder from 1 to 1 + digitCount.
      iii. Let index be ℝ(StringToNumber(digits)).
      iv. Assert: 0 ≤ index ≤ 99.
      v. Let captureLen be the number of elements in captures.
      vi. If index > captureLen and digitCount > 1, then
        1. NOTE: When a two-digit replacement pattern specifies an index exceeding the count of capturing groups, it is treated as a one-digit replacement pattern followed by a literal digit.
        2. Set digitCount to 1.
        3. Set digits to the substring of digits from 0 to 1.
        4. Set index to ℝ(StringToNumber(digits)).
      vii. Let ref be the substring of templateRemainder from 0 to 1 + digitCount.
      viii. If 1 ≤ index ≤ captureLen, then
        1. Let capture be captures[index - 1].
        2. If capture is undefined, then
          a. Let refReplacement be the empty String.
        3. Else,
          a. Let refReplacement be capture.
      ix. Else,
        1. Let refReplacement be ref.
    ...
    i. Let refLength be the length of ref.
    j. Set templateRemainder to the substring of templateRemainder from refLength.
    k. Set result to the string-concatenation of result and refReplacement.
  6. Return result.
---*/

var str = 'foo-x-bar';

var x = 'x';
var re0 = /x/;
var re1 = /(x)/;
var re1x = /(x)($^)?/;
var re10 = /((((((((((x))))))))))/;

assert.sameValue(str.replace(x, '|$0|'), 'foo-|$0|-bar',
  '`$0` is not a capture index for string "' + x + '"');
assert.sameValue(str.replace(re0, '|$0|'), 'foo-|$0|-bar',
  '`$0` is not a capture index in ' + String(re0));
assert.sameValue(str.replace(re1, '|$0|'), 'foo-|$0|-bar',
  '`$0` is not a capture index in ' + String(re1));
assert.sameValue(str.replace(re1x, '|$0|'), 'foo-|$0|-bar',
  '`$0` is not a capture index in ' + String(re1x));
assert.sameValue(str.replace(re10, '|$0|'), 'foo-|$0|-bar',
  '`$0` is not a capture index in ' + String(re10));

assert.sameValue(str.replace(x, '|$00|'), 'foo-|$00|-bar',
  '`$00` is not a capture index for string "' + x + '"');
assert.sameValue(str.replace(re0, '|$00|'), 'foo-|$00|-bar',
  '`$00` is not a capture index in ' + String(re0));
assert.sameValue(str.replace(re1, '|$00|'), 'foo-|$00|-bar',
  '`$00` is not a capture index in ' + String(re1));
assert.sameValue(str.replace(re1x, '|$00|'), 'foo-|$00|-bar',
  '`$00` is not a capture index in ' + String(re1x));
assert.sameValue(str.replace(re10, '|$00|'), 'foo-|$00|-bar',
  '`$00` is not a capture index in ' + String(re10));

assert.sameValue(str.replace(x, '|$000|'), 'foo-|$000|-bar',
  '`$00` before `0` is not a capture index for string "' + x + '"');
assert.sameValue(str.replace(re0, '|$000|'), 'foo-|$000|-bar',
  '`$00` before `0` is not a capture index in ' + String(re0));
assert.sameValue(str.replace(re1, '|$000|'), 'foo-|$000|-bar',
  '`$00` before `0` is not a capture index in ' + String(re1));
assert.sameValue(str.replace(re1x, '|$000|'), 'foo-|$000|-bar',
  '`$00` before `0` is not a capture index in ' + String(re1x));
assert.sameValue(str.replace(re10, '|$000|'), 'foo-|$000|-bar',
  '`$00` before `0` is not a capture index in ' + String(re10));

assert.sameValue(str.replace(x, '|$1|'), 'foo-|$1|-bar',
  '`$1` is not a capture index for string "' + x + '"');
assert.sameValue(str.replace(re0, '|$1|'), 'foo-|$1|-bar',
  '`$1` is not a capture index in ' + String(re0));
assert.sameValue(str.replace(re1, '|$1|'), 'foo-|x|-bar',
  '`$1` is a capture index in ' + String(re1));
assert.sameValue(str.replace(re1x, '|$1|'), 'foo-|x|-bar',
  '`$1` is a capture index in ' + String(re1x));
assert.sameValue(str.replace(re10, '|$1|'), 'foo-|x|-bar',
  '`$1` is a capture index in ' + String(re10));

assert.sameValue(str.replace(x, '|$01|'), 'foo-|$01|-bar',
  '`$01` is not a capture index for string "' + x + '"');
assert.sameValue(str.replace(re0, '|$01|'), 'foo-|$01|-bar',
  '`$01` is not a capture index in ' + String(re0));
assert.sameValue(str.replace(re1, '|$01|'), 'foo-|x|-bar',
  '`$01` is a capture index in ' + String(re1));
assert.sameValue(str.replace(re1x, '|$01|'), 'foo-|x|-bar',
  '`$01` is a capture index in ' + String(re1x));
assert.sameValue(str.replace(re10, '|$01|'), 'foo-|x|-bar',
  '`$01` is a capture index in ' + String(re10));

assert.sameValue(str.replace(x, '|$010|'), 'foo-|$010|-bar',
  '`$01` before `0` is not a capture index for string "' + x + '"');
assert.sameValue(str.replace(re0, '|$010|'), 'foo-|$010|-bar',
  '`$01` before `0` is not a capture index in ' + String(re0));
assert.sameValue(str.replace(re1, '|$010|'), 'foo-|x0|-bar',
  '`$01` before `0` is a capture index in ' + String(re1));
assert.sameValue(str.replace(re1x, '|$010|'), 'foo-|x0|-bar',
  '`$01` before `0` is a capture index in ' + String(re1x));
assert.sameValue(str.replace(re10, '|$010|'), 'foo-|x0|-bar',
  '`$01` before `0` is a capture index in ' + String(re10));

assert.sameValue(str.replace(x, '|$2|'), 'foo-|$2|-bar',
  '`$2` is not a capture index for string "' + x + '"');
assert.sameValue(str.replace(re0, '|$2|'), 'foo-|$2|-bar',
  '`$2` is not a capture index in ' + String(re0));
assert.sameValue(str.replace(re1, '|$2|'), 'foo-|$2|-bar',
  '`$2` is not a capture index in ' + String(re1));
assert.sameValue(str.replace(re1x, '|$2|'), 'foo-||-bar',
  '`$2` is a failed capture index in ' + String(re1x));
assert.sameValue(str.replace(re10, '|$2|'), 'foo-|x|-bar',
  '`$2` is a capture index in ' + String(re10));

assert.sameValue(str.replace(x, '|$02|'), 'foo-|$02|-bar',
  '`$02` is not a capture index for string "' + x + '"');
assert.sameValue(str.replace(re0, '|$02|'), 'foo-|$02|-bar',
  '`$02` is not a capture index in ' + String(re0));
assert.sameValue(str.replace(re1, '|$02|'), 'foo-|$02|-bar',
  '`$02` is not a capture index in ' + String(re1));
assert.sameValue(str.replace(re1x, '|$02|'), 'foo-||-bar',
  '`$02` is a failed capture index in ' + String(re1x));
assert.sameValue(str.replace(re10, '|$02|'), 'foo-|x|-bar',
  '`$02` is a capture index in ' + String(re10));

assert.sameValue(str.replace(x, '|$020|'), 'foo-|$020|-bar',
  '`$02` before `0` is not a capture index for string "' + x + '"');
assert.sameValue(str.replace(re0, '|$020|'), 'foo-|$020|-bar',
  '`$02` before `0` is not a capture index in ' + String(re0));
assert.sameValue(str.replace(re1, '|$020|'), 'foo-|$020|-bar',
  '`$02` before `0` is not a capture index in ' + String(re1));
assert.sameValue(str.replace(re1x, '|$020|'), 'foo-|0|-bar',
  '`$02` before `0` is a failed capture index in ' + String(re1x));
assert.sameValue(str.replace(re10, '|$020|'), 'foo-|x0|-bar',
  '`$02` before `0` is a capture index in ' + String(re10));

assert.sameValue(str.replace(x, '|$10|'), 'foo-|$10|-bar',
  '`$10` is not a capture index (nor is `$1`) for string "' + x + '"');
assert.sameValue(str.replace(re0, '|$10|'), 'foo-|$10|-bar',
  '`$10` is not a capture index (nor is `$1`) in ' + String(re0));
assert.sameValue(str.replace(re1, '|$10|'), 'foo-|x0|-bar',
  '`$10` is not a capture index (but `$1` is) in ' + String(re1));
assert.sameValue(str.replace(re1x, '|$10|'), 'foo-|x0|-bar',
  '`$10` is not a capture index (but `$1` is) in ' + String(re1x));
assert.sameValue(str.replace(re10, '|$10|'), 'foo-|x|-bar',
  '`$10` is a capture index in ' + String(re10));

assert.sameValue(str.replace(x, '|$100|'), 'foo-|$100|-bar',
  '`$10` before `0` is not a capture index (nor is `$1`) for string "' + x + '"');
assert.sameValue(str.replace(re0, '|$100|'), 'foo-|$100|-bar',
  '`$10` before `0` is not a capture index (nor is `$1`) in ' + String(re0));
assert.sameValue(str.replace(re1, '|$100|'), 'foo-|x00|-bar',
  '`$10` before `0` is not a capture index (but `$1` is) in ' + String(re1));
assert.sameValue(str.replace(re1x, '|$100|'), 'foo-|x00|-bar',
  '`$10` before `0` is not a capture index (but `$1` is) in ' + String(re1x));
assert.sameValue(str.replace(re10, '|$100|'), 'foo-|x0|-bar',
  '`$10` before `0` is a capture index in ' + String(re10));

assert.sameValue(str.replace(x, '|$20|'), 'foo-|$20|-bar',
  '`$20` is not a capture index (nor is `$2`) for string "' + x + '"');
assert.sameValue(str.replace(re0, '|$20|'), 'foo-|$20|-bar',
  '`$20` is not a capture index (nor is `$2`) in ' + String(re0));
assert.sameValue(str.replace(re1, '|$20|'), 'foo-|$20|-bar',
  '`$20` is not a capture index (nor is `$2`) in ' + String(re1));
assert.sameValue(str.replace(re1x, '|$20|'), 'foo-|0|-bar',
  '`$20` is not a capture index (but `$2` is a failed capture index) in ' + String(re1x));
assert.sameValue(str.replace(re10, '|$20|'), 'foo-|x0|-bar',
  '`$20` is not a capture index (but `$2` is) in ' + String(re10));

assert.sameValue(str.replace(x, '|$200|'), 'foo-|$200|-bar',
  '`$20` before `0` is not a capture index (nor is `$2`) for string "' + x + '"');
assert.sameValue(str.replace(re0, '|$200|'), 'foo-|$200|-bar',
  '`$20` before `0` is not a capture index (nor is `$2`) in ' + String(re0));
assert.sameValue(str.replace(re1, '|$200|'), 'foo-|$200|-bar',
  '`$20` before `0` is not a capture index (nor is `$2`) in ' + String(re1));
assert.sameValue(str.replace(re1x, '|$200|'), 'foo-|00|-bar',
  '`$20` before `0` is not a capture index (but `$2` is a failed capture index) in ' + String(re1x));
assert.sameValue(str.replace(re10, '|$200|'), 'foo-|x00|-bar',
  '`$20` before `0` is not a capture index (but `$2` is) in ' + String(re10));

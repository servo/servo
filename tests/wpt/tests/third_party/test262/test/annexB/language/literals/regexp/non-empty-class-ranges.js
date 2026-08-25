// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regular-expressions-patterns
es6id: B.1.4
description: Extensions to NonemptyClassRanges production
info: |
    The production NonemptyClassRanges :: ClassAtom-ClassAtom ClassRanges
    evaluates as follows:

    1. Evaluate the first ClassAtom to obtain a CharSet A.
    2. Evaluate the second ClassAtom to obtain a CharSet B.
    3. Evaluate ClassRanges to obtain a CharSet C.
    4. Call CharacterRangeOrUnion(A, B) and let D be the resulting CharSet.
    5. Return the union of CharSets D and C.

    B.1.4.1.1 Runtime Semantics: CharacterRangeOrUnion Abstract Operation

    1. If Unicode is false, then
       a. If A does not contain exactly one character or B does not contain
          exactly one character, then
          i. Let C be the CharSet containing the single character - U+002D
             (HYPHEN-MINUS).
          ii. Return the union of CharSets A, B and C.
    2. Return CharacterRange(A, B).
---*/

var match;

match = /[--\d]+/.exec('.-0123456789-.');
assert.sameValue(match[0], '-0123456789-');

match = /[--\dz]+/.exec('.-0123456789z-.');
assert.sameValue(match[0], '-0123456789z-');

// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.2.4
description: >
  Returns the string value appending the substitutions on the same index order.
info: |
  21.1.2.4 String.raw ( template , ...substitutions )

  ...
  10. Let stringElements be a new List.
  11. Let nextIndex be 0.
  12. Repeat
    a. Let nextKey be ToString(nextIndex).
    b. Let nextSeg be ToString(Get(raw, nextKey)).
    c. ReturnIfAbrupt(nextSeg).
    d. Append in order the code unit elements of nextSeg to the end of
    stringElements.
    e. If nextIndex + 1 = literalSegments, then
      i. Return the String value whose code units are, in order, the elements in
      the List stringElements. If stringElements has no elements, the empty
      string is returned.
    f. If nextIndex < numberOfSubstitutions, let next be substitutions[nextIndex].
    g. Else, let next be the empty String.
    h. Let nextSub be ToString(next).
    i. ReturnIfAbrupt(nextSub).
    j. Append in order the code unit elements of nextSub to the end of stringElements.
    k. Let nextIndex be nextIndex + 1.
---*/

var template = {
  raw: ['a', 'b', 'd', 'f']
};

assert.sameValue(String.raw(template, '', 'c', 'e'), 'abcdef');
assert.sameValue(String.raw(template, 1), 'a1bdf');

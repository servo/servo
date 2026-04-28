// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.2.4
description: >
  Limit appended substitutions arguments to template.raw.length - 1.
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
    ...
---*/

var template = {
  raw: ['a', 'c', 'e']
};
var obj = {
  toString: function() {
    throw new Test262Error();
  }
};

assert.sameValue(String.raw(template, 'b', 'd', obj), 'abcde');

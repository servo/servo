// Copyright 2019 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The properties of the "indices" array correspond to the start/end indices of the same values in the match.
includes: [compareArray.js]
esid: sec-makeindicesarray
features: [regexp-match-indices]
info: |
  MakeIndicesArray ( S, indices, groupNames, hasIndices )
    4. Let _n_ be the number of elements in _indices_.
    ...
    8. Set _A_ to ! ArrayCreate(_n_).
    ...
    13. For each integer _i_ such that _i_ >= 0 and _i_ < _n_, do
      a. Let _matchIndices_ be _indices_[_i_].
      b. If _matchIndices_ is not *undefined*, then
        i. Let _matchIndicesArray_ be ! GetMatchIndicesArray(_S_, _matchIndices_).
      c. Else,
        i. Let _matchIndicesArray_ be *undefined*.
      d. Perform ! CreateDataProperty(_A_, ! ToString(_n_), _matchIndicesArray_).
        ...
---*/

let input = "abcd";
let match = /b(c)/d.exec(input);
let indices = match.indices;

// `indices` has the same length as match
assert.sameValue(indices.length, match.length);

// The first element of `indices` contains the start/end indices of the match
assert.compareArray(indices[0], [1, 3]);
assert.sameValue(input.slice(indices[0][0], indices[0][1]), match[0]);

// The second element of `indices` contains the start/end indices of the first capture
assert.compareArray(indices[1], [2, 3]);
assert.sameValue(input.slice(indices[1][0], indices[1][1]), match[1]);

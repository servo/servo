// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.lastindexof
description: >
    String.prototype.lastIndexOf must be able to return -1
info: |
    String.prototype.lastIndexOf ( _searchString_ [ , _position_ ] )

    12. Let _result_ be StringLastIndexOf(_S_, _searchStr_, _start_).

    StringLastIndexOf (_string_, _searchValue_, _fromIndex_)

    4. For each integer _i_ such that 0 ≤ _i_ ≤ _fromIndex_, in descending order, do
        a. Let _candidate_ be the substring of _string_ from _i_ to _i_ + _searchLen_.
        b. If _candidate_ is _searchValue_, return _i_.
    5. Return ~not-found~.
---*/

assert.sameValue("abc".lastIndexOf("d"), -1, "String.prototype.lastIndexOf returns -1 when searchString is shorter than this and searchString is not a substring of this.");

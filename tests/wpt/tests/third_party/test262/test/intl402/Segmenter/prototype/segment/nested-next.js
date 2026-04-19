// Copyright 2020 the V8 project authors, Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%segmentsprototype%-@@iterator
description: Test to ensure the nested calling of the next method won't caused confusion to each other.
info: |
    %Segments.prototype% [ @@iterator ] ()
    5. Return ! CreateSegmentIterator(segmenter, string)

    CreateSegmentIterator ( segmenter, string )
    1. Let internalSlotsList be « [[IteratingSegmenter]], [[IteratedString]], [[IteratedStringNextSegmentCodeUnitIndex]] ».
    2. Let iterator be ! ObjectCreate(%SegmentIterator.prototype%, internalSlotsList).
    3. Set iterator.[[IteratingSegmenter]] to segmenter.
    4. Set iterator.[[IteratedString]] to string.
    5. Set iterator.[[IteratedStringNextSegmentCodeUnitIndex]] to 0.
    6. Return iterator.

    %SegmentIterator.prototype%.next ()
    5. Let startIndex be iterator.[[IteratedStringNextSegmentCodeUnitIndex]].

features: [Intl.Segmenter]
---*/

const segmenter = new Intl.Segmenter();
const input = "ABCD";
const segments = segmenter.segment(input);
let result = "";
for (let v1 of segments) {
  for (let v2 of segments) {
    result += v1.segment;
    result += v2.segment;
  }
  result += ":";
}
assert.sameValue("AAABACAD:BABBBCBD:CACBCCCD:DADBDCDD:", result);

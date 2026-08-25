// Copyright 2020 the V8 project authors, Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%segmentsprototype%-@@iterator
description: Test to ensure the next on two segments of the segmenter won't interfer each other.
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
const input1 = "ABCD";
const input2 = "123";
const segments1 = segmenter.segment(input1);
const segments2 = segmenter.segment(input2);
let result = "";
for (let v1 of segments1) {
  for (let v2 of segments2) {
    result += v1.segment;
    result += v2.segment;
  }
  result += ":";
}
// Now loop segments2 .
for (let v2 of segments2) {
  for (let v1 of segments1) {
    result += v2.segment;
    result += v1.segment;
  }
  result += ":";
}
assert.sameValue(
    "A1A2A3:B1B2B3:C1C2C3:D1D2D3:1A1B1C1D:2A2B2C2D:3A3B3C3D:", result);

// Copyright 2020 the V8 project authors, Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%segmentsprototype%-@@iterator
description: Test to ensure the the calling of containing() won't impact the calling of the next().
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

    %Segments.prototype%.containing ( index )
    3. Let segmenter be segments.[[SegmentsSegmenter]].
    4. Let string be segments.[[SegmentsString]].


features: [Intl.Segmenter]
---*/

const segmenter = new Intl.Segmenter();
const input = "ABC";
const segments = segmenter.segment(input);
let next_result = "";
for (let i = 0; i < input.length; i++) {
  let containing_result = segments.containing(i);
  let msg = "containing(" + i + ") before the loop. ";
  assert.sameValue(input[i], containing_result.segment, msg + "segment");
  assert.sameValue(i, containing_result.index, msg + "index");
  assert.sameValue(input, containing_result.input, msg + "input");
  for (let v of segments) {
    next_result += v.segment;
    next_result += ":";
    // Ensure the value n passing into segments.containing(n) will not impact
    // the result of next().
    msg = "containing(" + i + ") inside the loop. ";
    containing_result = segments.containing(i);
    assert.sameValue(input[i], containing_result.segment, msg + "segment");
    assert.sameValue(i, containing_result.index, msg + "index");
    assert.sameValue(input, containing_result.input, msg + "input");
  }
}
assert.sameValue("A:B:C:A:B:C:A:B:C:", next_result);

'use strict';

const validBinaryTypes = ['blob', 'arraybuffer'];
const invalidBinaryTypes = ['jellyfish', 'arraybuffer ', '', null, undefined];

for (const binaryType of validBinaryTypes) {
  test((t) => {
    const pc = new RTCPeerConnection();
    t.add_cleanup(() => pc.close());
    const dc = pc.createDataChannel('test-binary-type');

    dc.binaryType = binaryType;
    assert_equals(dc.binaryType, binaryType, `dc.binaryType should be '${binaryType}'`);
  }, `Setting binaryType to '${binaryType}' should succeed`);
}

for (const binaryType of invalidBinaryTypes) {
  test((t) => {
    const pc = new RTCPeerConnection();
    t.add_cleanup(() => pc.close());
    const dc = pc.createDataChannel('test-binary-type');

    assert_throws_dom('SyntaxError', () => {
      dc.binaryType = binaryType;
    });
  }, `Setting invalid binaryType '${binaryType}' should throw SyntaxError`);
}

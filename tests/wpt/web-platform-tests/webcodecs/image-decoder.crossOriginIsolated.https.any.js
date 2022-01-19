// META: global=window,dedicatedworker
// META: script=/webcodecs/image-decoder-utils.js

function testSharedArrayBuffer(useView) {
  const mimeType = 'image/png';
  var decoder = null;
  return ImageDecoder.isTypeSupported(mimeType).then(support => {
    assert_implements_optional(
        support, 'Optional codec ' + mimeType + ' not supported.');
    return fetch('four-colors.png').then(response => {
      return response.arrayBuffer().then(buffer => {
        let data = new SharedArrayBuffer(buffer.byteLength);
        let view = new Uint8Array(data);
        view.set(new Uint8Array(buffer));
        return testFourColorsDecodeBuffer(useView ? view : data, mimeType);
      });
    });
  });
}

promise_test(t => {
  return testSharedArrayBuffer(/*useView=*/ false);
}, 'Test ImageDecoder decoding with a SharedArrayBuffer source');

promise_test(t => {
  return testSharedArrayBuffer(/*useView=*/ true);
}, 'Test ImageDecoder decoding with a Uint8Array(SharedArrayBuffer) source');

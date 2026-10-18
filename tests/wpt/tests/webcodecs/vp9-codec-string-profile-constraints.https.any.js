// META: global=window,dedicatedworker

// The VP9 codec string is defined by the "Codecs Parameter String" section of
// the VP Codec ISO Media File Format Binding, which constrains bit depth and
// chroma subsampling by profile: profile 0 is 8-bit 4:2:0, profile 1 is 8-bit
// 4:2:2/4:4:4, profile 2 is 10/12-bit 4:2:0 and profile 3 is 10/12-bit
// 4:2:2/4:4:4. A codec string that does not meet these constraints is not a
// valid codec string, so Check Configuration Support returns false.
const invalidCodecs = [
  {comment: 'missing level and bit depth', codec: 'vp09.00'},
  {comment: 'missing bit depth', codec: 'vp09.00.10'},
  {comment: 'invalid bit depth', codec: 'vp09.00.10.09'},
  {comment: 'profile 0 with 10-bit', codec: 'vp09.00.10.10'},
  {comment: 'profile 0 with 12-bit', codec: 'vp09.00.10.12'},
  {comment: 'profile 0 with 4:2:2', codec: 'vp09.00.10.08.02'},
  {comment: 'profile 0 with 4:4:4', codec: 'vp09.00.10.08.03'},
  {comment: 'profile 2 with 8-bit', codec: 'vp09.02.10.08'},
  {comment: 'profile 2 with 4:2:2', codec: 'vp09.02.10.10.02'},
  {comment: 'profile 2 with 4:4:4', codec: 'vp09.02.10.10.03'},
];

invalidCodecs.forEach(entry => {
  promise_test(async t => {
    const support = await VideoDecoder.isConfigSupported({codec: entry.codec});
    assert_false(support.supported);
  }, `VideoDecoder.isConfigSupported() doesn't support ${entry.codec} (${entry.comment})`);

  promise_test(async t => {
    const support = await VideoEncoder.isConfigSupported(
        {codec: entry.codec, width: 320, height: 240});
    assert_false(support.supported);
  }, `VideoEncoder.isConfigSupported() doesn't support ${entry.codec} (${entry.comment})`);
});

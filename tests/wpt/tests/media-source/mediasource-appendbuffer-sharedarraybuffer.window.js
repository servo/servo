// META: title=SourceBuffer.appendBuffer() shared buffer test cases
// META: script=mediasource-util.js

// The data argument of appendBuffer() is not [AllowShared], so a
// SharedArrayBuffer, or a view onto one, has to throw a TypeError.
//
// See https://github.com/whatwg/html/issues/5380 for why not `new SharedArrayBuffer()`.
const sharedBuffer = new WebAssembly.Memory({ shared: true, initial: 1, maximum: 1 }).buffer;

for (const [kind, data] of [["SharedArrayBuffer", sharedBuffer],
                            ["Uint8Array(SharedArrayBuffer)", new Uint8Array(sharedBuffer)]]) {
    mediasource_test((test, mediaElement, mediaSource) => {
        const sourceBuffer = mediaSource.addSourceBuffer(MediaSourceUtil.AUDIO_VIDEO_TYPE);
        assert_throws_js(TypeError, () => sourceBuffer.appendBuffer(data),
                         `appendBuffer() threw an exception when passed a ${kind}.`);
        test.done();
    }, `Test appendBuffer() with a ${kind}`);
}

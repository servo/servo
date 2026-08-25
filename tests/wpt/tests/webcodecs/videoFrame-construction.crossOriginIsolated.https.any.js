// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js
// META: script=/webcodecs/videoFrame-utils.js

test(t => {
  testBufferConstructedI420Frame('SharedArrayBuffer');
}, 'Test SharedArrayBuffer constructed I420 VideoFrame');

test(t => {
  testBufferConstructedI420Frame('Uint8Array(SharedArrayBuffer)');
}, 'Test Uint8Array(SharedArrayBuffer) constructed I420 VideoFrame');

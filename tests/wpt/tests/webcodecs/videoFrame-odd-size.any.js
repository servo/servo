// META: global=window,dedicatedworker
// META: script=/webcodecs/videoFrame-utils.js

test(t => {
  let fmt = 'I420';
  let vfInit = {
    format: fmt,
    timestamp: 1234,
    codedWidth: 3,
    codedHeight: 3,
    visibleRect: {x: 0, y: 0, width: 3, height: 3},
  };
  let data = new Uint8Array([
    1, 2, 3, 4, 5, 6, 7, 8, 9,  // y
    1, 2, 3, 4,                 // u
    1, 2, 3, 4,                 // v
  ]);
  let frame = new VideoFrame(data, vfInit);
  assert_equals(frame.format, fmt, 'format');
  assert_equals(frame.visibleRect.x, 0, 'visibleRect.x');
  assert_equals(frame.visibleRect.y, 0, 'visibleRect.y');
  assert_equals(frame.visibleRect.width, 3, 'visibleRect.width');
  assert_equals(frame.visibleRect.height, 3, 'visibleRect.height');
  frame.close();
}, 'Test I420 VideoFrame construction with odd coded size');

promise_test(async t => {
  const init = {
    format: 'I420',
    timestamp: 0,
    codedWidth: 3,
    codedHeight: 3,
  };
  const buf = new Uint8Array([
    1, 2, 3, 4, 5, 6, 7, 8, 9,  // y
    10, 11, 12, 13,             // u
    14, 15, 16, 17,             // v
  ]);
  const expectedLayout = [
    {offset: 0, stride: 3},
    {offset: 9, stride: 2},
    {offset: 13, stride: 2},
  ];
  const frame = new VideoFrame(buf, init);
  assert_equals(frame.allocationSize(), buf.length, 'allocationSize()');
  const data = new Uint8Array(buf.length);
  const layout = await frame.copyTo(data);
  assert_layout_equals(layout, expectedLayout);
  assert_buffer_equals(data, buf);
}, 'Test I420 copyTo with odd coded size.');

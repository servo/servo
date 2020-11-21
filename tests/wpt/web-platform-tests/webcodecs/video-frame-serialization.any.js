// META: global=window,dedicatedworker
// META: script=/common/media.js
// META: script=/webcodecs/utils.js

var defaultInit = {
  timestamp : 100,
  duration : 33,
}

function createDefaultVideoFrame() {
  let image = makeImageBitmap(32,16);

  return new VideoFrame(image, defaultInit);
}

test(t => {
  let frame = createDefaultVideoFrame();

  let clone = frame.clone();

  assert_equals(frame.timestamp, clone.timestamp);
  assert_equals(frame.duration, clone.duration);
  assert_equals(frame.cropWidth, clone.cropWidth);
  assert_equals(frame.cropHeight, clone.cropHeight);
  assert_equals(frame.cropWidth, clone.cropWidth);
  assert_equals(frame.cropHeight, clone.cropHeight);

  frame.destroy();
  clone.destroy();
}, 'Test we can clone a VideoFrame.');

test(t => {
  let frame = createDefaultVideoFrame();

  let copy = frame;
  let clone = frame.clone();

  frame.destroy();

  assert_not_equals(copy.timestamp, defaultInit.timestamp);
  assert_equals(clone.timestamp, defaultInit.timestamp);

  clone.destroy();
}, 'Verify destroying a frame doesn\'t affect its clones.');

test(t => {
  let frame = createDefaultVideoFrame();

  frame.destroy();

  assert_throws_dom("InvalidStateError", () => {
    let clone = frame.clone();
  });
}, 'Verify cloning a destroyed frame throws.');

async_test(t => {
  let localFrame = createDefaultVideoFrame();

  let channel = new MessageChannel();
  let localPort = channel.port1;
  let externalPort = channel.port2;

  externalPort.onmessage = t.step_func((e) => {
    let externalFrame = e.data;
    externalFrame.destroy();
    externalPort.postMessage("Done");
  })

  localPort.onmessage = t.step_func_done((e) => {
    assert_not_equals(localFrame.timestamp, defaultInit.timestamp);
  })

  localPort.postMessage(localFrame);

}, 'Verify destroying frames propagates accross contexts.');

async_test(t => {
  let localFrame = createDefaultVideoFrame();

  let channel = new MessageChannel();
  let localPort = channel.port1;
  let externalPort = channel.port2;

  externalPort.onmessage = t.step_func((e) => {
    let externalFrame = e.data;
    externalFrame.destroy();
    externalPort.postMessage("Done");
  })

  localPort.onmessage = t.step_func_done((e) => {
    assert_equals(localFrame.timestamp, defaultInit.timestamp);
    localFrame.destroy();
  })

  localPort.postMessage(localFrame.clone());

}, 'Verify destroying cloned frames doesn\'t propagate accross contexts.');

async_test(t => {
  let localFrame = createDefaultVideoFrame();

  let channel = new MessageChannel();
  let localPort = channel.port1;

  localPort.onmessage = t.unreached_func();

  localFrame.destroy();

  assert_throws_dom("DataCloneError", () => {
    localPort.postMessage(localFrame);
  });

  t.done();
}, 'Verify posting destroyed frames throws.');

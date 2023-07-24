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
  assert_equals(frame.visibleRect.left, clone.visibleRect.left);
  assert_equals(frame.visibleRect.top, clone.visibleRect.top);
  assert_equals(frame.visibleRect.width, clone.visibleRect.width);
  assert_equals(frame.visibleRect.height, clone.visibleRect.height);

  frame.close();
  assert_true(isFrameClosed(frame));
  clone.close();
  assert_true(isFrameClosed(clone));
}, 'Test we can clone a VideoFrame.');

test(t => {
  let frame = createDefaultVideoFrame();

  let copy = frame;
  let clone = frame.clone();

  frame.close();

  assert_equals(copy.timestamp, defaultInit.timestamp);
  assert_equals(copy.duration, defaultInit.duration);
  assert_true(isFrameClosed(copy));
  assert_equals(clone.timestamp, defaultInit.timestamp);
  assert_false(isFrameClosed(clone));

  clone.close();
}, 'Verify closing a frame doesn\'t affect its clones.');

test(t => {
  let frame = createDefaultVideoFrame();

  frame.close();

  assert_throws_dom("InvalidStateError", () => {
    let clone = frame.clone();
  });
}, 'Verify cloning a closed frame throws.');

async_test(t => {
  let localFrame = createDefaultVideoFrame();

  let channel = new MessageChannel();
  let localPort = channel.port1;
  let externalPort = channel.port2;

  externalPort.onmessage = t.step_func((e) => {
    let externalFrame = e.data;
    externalFrame.close();
    externalPort.postMessage("Done");
  })

  localPort.onmessage = t.step_func_done((e) => {
    assert_equals(localFrame.timestamp, defaultInit.timestamp);
    localFrame.close();
  })

  localPort.postMessage(localFrame);
}, 'Verify closing frames does not propagate accross contexts.');

async_test(t => {
  let localFrame = createDefaultVideoFrame();

  let channel = new MessageChannel();
  let localPort = channel.port1;
  let externalPort = channel.port2;

  externalPort.onmessage = t.step_func_done((e) => {
    let externalFrame = e.data;
    assert_equals(externalFrame.timestamp, defaultInit.timestamp);
    externalFrame.close();
  })

  localPort.postMessage(localFrame, [localFrame]);
  assert_true(isFrameClosed(localFrame));
}, 'Verify transferring frames closes them.');

async_test(t => {
  let localFrame = createDefaultVideoFrame();

  let channel = new MessageChannel();
  let localPort = channel.port1;

  localPort.onmessage = t.unreached_func();

  localFrame.close();

  assert_throws_dom("DataCloneError", () => {
    localPort.postMessage(localFrame);
  });

  t.done();
}, 'Verify posting closed frames throws.');

promise_test(async t => {
  const open = indexedDB.open('VideoFrameTestDB', 1);
  open.onerror = t.unreached_func('open should succeed');
  open.onupgradeneeded = (event) => {
    let db = event.target.result;
    db.createObjectStore('MyVideoFrames', { keyPath: 'id' });
  };
  let db = await new Promise((resolve) => {
    open.onsuccess = (e) => {
      resolve(e.target.result);
    };
  });
  t.add_cleanup(() => {
    db.close();
    indexedDB.deleteDatabase(db.name);
  });

  let transaction = db.transaction(['MyVideoFrames'], 'readwrite');
  const store = transaction.objectStore('MyVideoFrames');
  let frame = createDefaultVideoFrame();
  assert_throws_dom("DataCloneError", () => {
    store.add(frame);
  });
}, 'Verify storing a frame throws.');

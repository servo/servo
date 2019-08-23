// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js

function taintedImageBitmap(t) {
  return new Promise(resolve => {
    const img = new Image();
    img.src = `${get_host_info().HTTPS_REMOTE_ORIGIN}/images/blue.png`;
    img.onload = t.step_func(() => {
      resolve(createImageBitmap(img));
    });
    img.onerror = t.unreached_func();
  });
}

async_test(t => {
  const bc = new BroadcastChannel(token());
  const popup = window.open(`resources/coop-coep-popup.html?channel=${bc.name}`);
  const popupReady = new Promise(resolve => {
    bc.onmessage = t.step_func(resolve);
  });
  const imageReady = taintedImageBitmap(t);
  Promise.all([popupReady, imageReady]).then(t.step_func(([, bitmap]) => {
    bc.onmessage = t.step_func_done(e => {
      assert_equals(e.data, "Got failure as expected.");
    });
    bc.postMessage(bitmap);
  }));
}, "BroadcastChannel'ing a tainted ImageBitmap to a COOP+COEP popup");

[
  {
    "type": "serialize/deserialize",
    "message": (port, bitmap) => port.postMessage(bitmap)
  },
  {
    "type": "transfer",
    "message": (port, bitmap) => port.postMessage(bitmap, [bitmap])
  }
].forEach(({ type, message }) => {
  async_test(t => {
    const sw = new SharedWorker("resources/coop-coep-worker.js");
    const imageReady = taintedImageBitmap(t);
    imageReady.then(t.step_func(bitmap => {
      sw.port.onmessage = t.step_func_done(e => {
        assert_equals(e.data, "Got failure as expected.");
      });
      message(sw.port, bitmap);
    }));
  }, `Messaging a tainted ImageBitMap via ${type} to a COEP shared worker`);
});

function loadSharedStorageImage(data) {
  let {key, value, hasSharedStorageWritableAttribute, isSameOrigin} = data;
  const encodedKey = encodeURIComponent(key);
  const encodedValue = encodeURIComponent(value);
  const sameOriginSrc = `/shared-storage/resources/` +
      `shared-storage-writable-pixel.png?key=${encodedKey}&value=${encodedValue}`;
  const crossOriginSrc =
      'https://{{domains[www]}}:{{ports[https][0]}}' + sameOriginSrc;

  let image = document.createElement('img');
  image.src = isSameOrigin ? sameOriginSrc : crossOriginSrc;
  if (hasSharedStorageWritableAttribute) {
    image.sharedStorageWritable = true;
  }

  const promise = new Promise((resolve, reject) => {
    image.addEventListener('load', () => {
      resolve(image);
    });
    image.addEventListener('error', () => {
      reject(new Error('Image load failed'));
    });
  });

  document.body.appendChild(image);
  return promise;
}

function navigateSharedStorageIframe(data) {
  let {
    hasSharedStorageWritableAttribute,
    rawWriteHeader,
    isSameOrigin,
    expectSharedStorageWritableHeader
  } = data;
  const writeHeader = encodeURIComponent(rawWriteHeader);
  const sameOriginSrc =
      `/shared-storage/resources/shared-storage-write-notify-parent.py` +
      `?write=${writeHeader}`;
  const crossOriginSrc =
      'https://{{domains[www]}}:{{ports[https][0]}}' + sameOriginSrc;

  let frame = document.createElement('iframe');
  frame.src = isSameOrigin ? sameOriginSrc : crossOriginSrc;
  if (hasSharedStorageWritableAttribute) {
    frame.sharedStorageWritable = true;
  }

  const expectedResult = expectSharedStorageWritableHeader ?
      '?1' :
      'NO_SHARED_STORAGE_WRITABLE_HEADER';

  function checkExpectedResult(data) {
    assert_equals(data.sharedStorageWritableHeader, expectedResult);
  }

  const promise = new Promise((resolve, reject) => {
    window.addEventListener('message', async function handler(evt) {
      if (evt.source === frame.contentWindow) {
        checkExpectedResult(evt.data);
        document.body.removeChild(frame);
        window.removeEventListener('message', handler);
        resolve();
      }
    });
    window.addEventListener('error', () => {
      reject(new Error('Navigation error'));
    });
  });

  document.body.appendChild(frame);
  return promise;
}

async function loadNestedSharedStorageFrameInNewFrame(data) {
  const SCOPE = '/shared-storage/resources/shared-storage-writ';
  const INTERMEDIATE_FRAME_SUFFIX =
      'able-fetch-request-fallback-to-network-iframe.https.html'
  const CROSS_ORIGIN = 'https://{{domains[www]}}:{{ports[https][0]}}';

  let {key, value, hasSharedStorageWritableAttribute, isSameOrigin} = data;

  const windowPromise = new Promise((resolve, reject) => {
    window.addEventListener('message', async function handler(evt) {
      if (evt.data.msg && evt.data.msg === 'iframe loaded') {
        window.removeEventListener('message', handler);
        resolve();
      }
    });
    window.addEventListener('error', () => {
      reject(new Error('Navigation error'));
    });
  });

  const framePromise = new Promise((resolve, reject) => {
    let frame = document.createElement('iframe');
    frame.src = SCOPE + INTERMEDIATE_FRAME_SUFFIX;
    frame.onload = function() {
      resolve(frame);
    };
    frame.onerror = function() {
      reject(new Error('Iframe load failed'));
    };
    document.body.appendChild(frame);
  });
  let frame = await framePromise;

  let rawWriteHeader = `set;key=${key};value=${value}`;
  let writeHeader = encodeURIComponent(rawWriteHeader);
  const sameOriginNestedSrc = `/shared-storage/resources/` +
      `shared-storage-write.py?write=${writeHeader}`;
  const nestedSrc =
      isSameOrigin ? sameOriginNestedSrc : CROSS_ORIGIN + sameOriginNestedSrc;

  let nestedFrame = frame.contentWindow.loadFrame(
      nestedSrc, hasSharedStorageWritableAttribute);
  await windowPromise;
  return {frame: frame, nestedFrame: nestedFrame, nestedFrameUrl: nestedSrc};
}

async function testCreateWorkletWithDataOption(
    test, data_origin, key, value, is_same_origin_script, expect_success) {
  const sameOrigin = location.origin;
  const crossOrigin = 'https://{{domains[www]}}:{{ports[https][0]}}';
  const sameOriginScriptUrl = `/shared-storage/resources/simple-module.js`;
  const scriptOrigin = is_same_origin_script ? sameOrigin : crossOrigin;
  const scriptUrl = is_same_origin_script ? sameOriginScriptUrl :
                                            crossOrigin + sameOriginScriptUrl;
  const dataOrigin =
      (data_origin === 'script-origin') ? scriptOrigin : sameOrigin;
  let success = false;
  let error = null;

  try {
    const worklet = await sharedStorage.createWorklet(
        scriptUrl, {credentials: 'omit', dataOrigin: data_origin});

    const ancestor_key = token();
    let url0 =
        generateURL('/shared-storage/resources/frame0.html', [ancestor_key]);

    let select_url_result =
        await worklet.selectURL('test-url-selection-operation', [{url: url0}], {
          data: {'mockResult': 0, 'setKey': key, 'setValue': value},
          resolveToConfig: true,
          keepAlive: true
        });

    assert_true(validateSelectURLResult(select_url_result, true));
    attachFencedFrame(select_url_result, 'opaque-ads');
    const result0 = await nextValueFromServer(ancestor_key);
    assert_equals(result0, 'frame0_loaded');

    await verifyKeyValueForOrigin(key, value, dataOrigin);
    await deleteKeyForOrigin(key, dataOrigin);
    success = true;
  } catch (e) {
    error = e;
    assert_equals(e.name, 'TypeError');
  } finally {
    assert_equals(
        expect_success, success,
        error ? 'expected success but error thrown: ' + error.toString() :
                'no error caught even though one was expected');
    test.done();
  }
}

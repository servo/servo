function loadSharedStorageImage(data) {
  let {key, value, hasSharedStorageWritableAttribute, isSameOrigin} = data;
  const sameOriginSrc = `/shared-storage/resources/` +
      `shared-storage-writable-pixel.png?key=${key}&value=${value}`;
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

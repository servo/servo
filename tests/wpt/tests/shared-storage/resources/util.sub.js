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

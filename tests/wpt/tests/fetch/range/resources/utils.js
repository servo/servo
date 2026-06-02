function loadScript(url, { doc = document }={}) {
  return new Promise((resolve, reject) => {
    const script = doc.createElement('script');
    script.onload = () => resolve();
    script.onerror = () => reject(Error("Script load failed"));
    script.src = url;
    doc.body.appendChild(script);
  })
}

function loadImage(url, { doc = document }={}) {
  return new Promise((resolve, reject) => {
    const img = doc.createElement('img');
    img.onload = () => resolve();
    img.onerror = () => reject(Error("Image load failed"));
    img.src = url;
    doc.body.appendChild(img);
  })
}

function preloadImage(url, { doc = document }={}) {
  return new Promise((resolve, reject) => {
    const preload = doc.createElement('link');
    preload.rel = 'preload';
    preload.as = 'image';
    preload.onload = () => resolve();
    preload.onerror = () => resolve();
    preload.href = url;
    doc.body.appendChild(preload);
  })
}

/**
 *
 * @param {Document} document
 * @param {string|URL} url
 * @returns {HTMLAudioElement}
 */
function appendAudio(document, url) {
  const audio = document.createElement('audio');
  audio.muted = true;
  audio.src = url;
  audio.preload = true;
  document.body.appendChild(audio);
  return audio;
}

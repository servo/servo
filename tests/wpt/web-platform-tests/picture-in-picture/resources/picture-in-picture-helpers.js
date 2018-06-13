function callWithTrustedClick(callback) {
  return new Promise(resolve => {
    let button = document.createElement('button');
    button.textContent = 'click to continue test';
    button.style.display = 'block';
    button.style.fontSize = '20px';
    button.style.padding = '10px';
    button.onclick = () => {
      document.body.removeChild(button);
      resolve(callback());
    };
    document.body.appendChild(button);
    test_driver.click(button);
  });
}

function loadVideo(activeDocument, sourceUrl) {
  return new Promise((resolve, reject) => {
    const document = activeDocument || window.document;
    const video = document.createElement('video');
    video.src = sourceUrl || '/media/movie_5.ogv';
    video.onloadedmetadata = () => {
      resolve(video);
    };
    video.onerror = error => {
      reject(error);
    };
  });
}

// Calls requestPictureInPicture() in a context that's 'allowed to request PiP'.
function requestPictureInPictureWithTrustedClick(videoElement) {
  return callWithTrustedClick(
      () => videoElement.requestPictureInPicture());
}

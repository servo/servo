if (!('pictureInPictureEnabled' in document)) {
  HTMLVideoElement.prototype.requestPictureInPicture = function() {
    return Promise.reject('Picture-in-Picture API is not available');
  }
}

function loadVideo(activeDocument, sourceUrl) {
  return new Promise((resolve, reject) => {
    const document = activeDocument || window.document;
    const video = document.createElement('video');
    video.src = sourceUrl || '/media/movie_5.ogv';
    video.onloadedmetadata = () => { resolve(video); };
    video.onerror = error => { reject(error); };
  });
}

// Calls requestPictureInPicture() in a context that's 'allowed to request PiP'.
function requestPictureInPictureWithTrustedClick(videoElement) {
  return test_driver.bless(
    'request Picture-in-Picture',
    () => videoElement.requestPictureInPicture());
}

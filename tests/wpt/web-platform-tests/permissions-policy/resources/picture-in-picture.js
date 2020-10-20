function async_pip_test(func, name) {
  async_test(t => {
    assert_true('pictureInPictureEnabled' in document, 'Picture-in-Picture API is available');
    func(t);
  }, name);
}

function promise_pip_test(func, name) {
  promise_test(async t => {
    assert_true('pictureInPictureEnabled' in document, 'Picture-in-Picture API is available');
    return func(t);
  }, name);
}

function isPictureInPictureAllowed() {
  return new Promise(resolve => {
    let video = document.createElement('video');
    video.src = getVideoURI('/media/movie_5');
    video.onloadedmetadata = () => {
      video.requestPictureInPicture()
      .then(() => resolve(document.pictureInPictureEnabled))
      .catch(e => {
        if (e.name == 'NotAllowedError')
          resolve(document.pictureInPictureEnabled);
        else
          resolve(false);
      });
    };
  });
}

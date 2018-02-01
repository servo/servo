function isPictureInPictureAllowed() {
  return new Promise((resolve, reject) => {
    const video = document.createElement('video');
    video.requestPictureInPicture()
    .then(() => resolve(document.pictureInPictureEnabled))
    .catch(e => {
      if (e.name == 'NotAllowedError')
        resolve(document.pictureInPictureEnabled);
      else
        resolve(false);
    });
  });
}
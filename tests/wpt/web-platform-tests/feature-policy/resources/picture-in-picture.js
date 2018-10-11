function isPictureInPictureAllowed() {
  if (!('pictureInPictureEnabled' in document))
    return Promise.resolve(false);

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

function isPictureInPictureAllowed() {
  return new Promise(resolve => {
    let video = document.createElement('video');
    video.src = '/media/movie_5.ogv';
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
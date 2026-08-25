promise_test(async t => {
  const track = new MediaStreamTrackGenerator('video');
  const capturer = new ImageCapture(track);
  const settings = await capturer.getPhotoSettings();
  await promise_rejects_dom(t, 'UnknownError', capturer.takePhoto(settings));
}, 'exercise takePhoto() on a track without PhotoCapabilities');

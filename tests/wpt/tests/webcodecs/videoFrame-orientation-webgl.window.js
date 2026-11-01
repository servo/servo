// META: script=/webcodecs/videoFrame-orientation-utils.js

runOrientationTests('texImage2D()', (frame, orientation, description) => {
  const {width, height} = orientedSize(orientation);
  verifyOrientedPixels(readTexturePixels(frame, width, height), orientation, description);
});

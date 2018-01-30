// This functions calls a callback with each API object as specified
// by https://immersive-web.github.io/webxr/spec/latest/, allowing
// checks to be made on all ojects.
// Arguements:
//      callback: A callback function with two arguements, the first
//                being the API object, the second being the name of
//                that API object.
function forEachWebxrObject(callback) {
  callback(window.navigator.xr, 'navigator.xr');
  callback(window.XRDevice, 'XRDevice');
  callback(window.XRSession, 'XRSession');
  callback(window.XRSessionCreationOptions, 'XRSessionCreationOptions');
  callback(window.XRFrameRequestCallback, 'XRFrameRequestCallback');
  callback(window.XRPresentationFrame, 'XRPresentationFrame');
  callback(window.XRView, 'XRView');
  callback(window.XRViewport, 'XRViewport');
  callback(window.XRDevicePose, 'XRDevicePose');
  callback(window.XRLayer, 'XRLayer');
  callback(window.XRWebGLLayer, 'XRWebGLLayer');
  callback(window.XRWebGLLayerInit, 'XRWebGLLayerInit');
  callback(window.XRCoordinateSystem, 'XRCoordinateSystem');
  callback(window.XRFrameOfReference, 'XRFrameOfReference');
  callback(window.XRStageBounds, 'XRStageBounds');
  callback(window.XRStageBoundsPoint, 'XRStageBoundsPoint');
  callback(window.XRSessionEvent, 'XRSessionEvent');
  callback(window.XRCoordinateSystemEvent, 'XRCoordinateSystemEvent');
}
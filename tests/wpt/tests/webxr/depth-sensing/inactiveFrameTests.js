'use strict';
const inactiveFrameTestFunction = function(session, controller, t, sessionObjects) {
  const isCpuOptimized = session.depthUsage === 'cpu-optimized';
  return session.requestReferenceSpace('viewer').then((viewerSpace) => {
    let callbacksKickedOff = false;
    let callbackCounter = 0;

    const glBinding = new XRWebGLBinding(session, sessionObjects.gl);

    const rafCb = function(time, frame) {
      const pose = frame.getViewerPose(viewerSpace);
      for(const view of pose.views) {
        const callback = () => {
          t.step(() => {
            assert_throws_dom("InvalidStateError",
                              () => isCpuOptimized ? frame.getDepthInformation(view)
                                                    : glBinding.getDepthInformation(view),
                              "getDepthInformation() should throw when ran outside RAF");
            assert_throws_dom("InvalidStateError",
                              () => session.pauseDepthSensing(),
                              "pauseDepthSensing() should thrown when ran outside RAF");
            assert_throws_dom("InvalidStateError",
                              () => session.resumeDepthSensing(),
                              "resumeDepthSensing() should thrown when ran outside RAF");
          });
          callbackCounter--;
        }

        t.step_timeout(callback, 10);
        callbackCounter++;
      }

      callbacksKickedOff = true;
    };

    session.requestAnimationFrame(rafCb);

    return t.step_wait(() => callbacksKickedOff && (callbackCounter == 0));
  });
};

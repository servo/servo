'use strict';

const staleViewsTestFunctionGenerator = function(isCpuOptimized) {
  return (session, controller, t, sessionObjects) => {
    let done = false;

    const staleViews = new Set();

    return session.requestReferenceSpace('viewer').then((viewerSpace) => {
      const glBinding = new XRWebGLBinding(session, sessionObjects.gl);

      const secondRafCb = function(time, frame) {
        for(const view of staleViews) {
          t.step(() => {
            assert_throws_dom("InvalidStateError",
                                () => isCpuOptimized ? frame.getDepthInformation(view)
                                                     : glBinding.getDepthInformation(view),
                                "getDepthInformation() should throw when run with stale XRView");
          });
        }

        done = true;
      };

      const firstRafCb = function(time, frame) {
        const pose = frame.getViewerPose(viewerSpace);
        for(const view of pose.views) {
          staleViews.add(view);
        }

        session.requestAnimationFrame(secondRafCb);
      };

      session.requestAnimationFrame(firstRafCb);

      return t.step_wait(() => done);
    });
  };
};
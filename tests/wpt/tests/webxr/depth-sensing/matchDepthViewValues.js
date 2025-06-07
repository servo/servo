'use strict';

// import * as XrConstants from 'resources/webxr_test_constants.js'
// import * as XrAsserts from 'resources/webxr_test_asserts.js'

// TODO: Expand the WebXrTestApi to specify a viewGeometry that this can validate
// as well.
const depthViewGeometryTestGenerator = (matchDepthView) => {
  return (session, controller, t, sessionObjects) => {

    return session.requestReferenceSpace('viewer').then((viewerSpace) => new Promise((resolve) => {

      const isCpuOptimized = session.depthUsage === 'cpu-optimized';
      const glBinding = new XRWebGLBinding(session, sessionObjects.gl);

      const rafCb = function(time, frame) {
        const pose = frame.getViewerPose(viewerSpace);
        for(const view of pose.views) {
          const depthInformation = isCpuOptimized ? frame.getDepthInformation(view)
                                                  : glBinding.getDepthInformation(view);
          if (matchDepthView) {
            t.step(()=> {
              assert_matrix_approx_equals(view.projectionMatrix, depthInformation.projectionMatrix);
              assert_transform_approx_equals(view.transform, depthInformation.transform);
            });
          } else {
            t.step(() => {
              assert_matrix_significantly_not_equals(view.projectionMatrix, depthInformation.projectionMatrix);
              assert_transform_significantly_not_equals(view.transform, depthInformation.transform);
            });
          }
        }
        resolve();
      }

      session.requestAnimationFrame(rafCb);
    })); // Promise
  }; // Test Func
};  // Generator Func

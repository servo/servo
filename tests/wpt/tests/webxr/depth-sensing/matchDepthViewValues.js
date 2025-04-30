'use strict';

// import * as XrConstants from 'resources/webxr_test_constants.js'
// import * as XrAsserts from 'resources/webxr_test_asserts.js'

// TODO: Expand the WebXrTestApi to specify a viewGeometry that this can validate
// as well.
const depthViewGeometryTestGenerator = function(isCpuOptimized) {
  return (session, controller, t, sessionObjects) => {

    return session.requestReferenceSpace('viewer').then((viewerSpace) => new Promise((resolve) => {

      const glBinding = new XRWebGLBinding(session, sessionObjects.gl);

      const rafCb = function(time, frame) {
        const pose = frame.getViewerPose(viewerSpace);
        for(const view of pose.views) {
          const depthInformation = isCpuOptimized ? frame.getDepthInformation(view)
                                                  : glBinding.getDepthInformation(view);
          t.step(()=> {
            assert_matrix_approx_equals(IDENTITY_MATRIX, depthInformation.projectionMatrix);
            assert_transform_approx_equals(IDENTITY_TRANSFORM, depthInformation.transform);
          });
        }
        resolve();
      }

      session.requestAnimationFrame(rafCb);
    })); // Promise
  }; // Test Func
};  // Generator Func

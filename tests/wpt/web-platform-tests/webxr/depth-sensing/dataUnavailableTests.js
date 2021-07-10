'use strict';

const TestStates = Object.freeze({
  "ShouldSucceedScheduleRAF": 1,
  "ShouldFailScheduleRAF": 2,
  "ShouldSucceedTestDone": 3,
});

const dataUnavailableTestFunctionGenerator = function(isCpuOptimized) {
  return (session, controller, t, sessionObjects) => {
    let state = TestStates.ShouldSucceedScheduleRAF;

    return session.requestReferenceSpace('viewer').then((viewerSpace) => {
      let done = false;

      const glBinding = new XRWebGLBinding(session, sessionObjects.gl);

      const rafCb = function(time, frame) {
        const pose = frame.getViewerPose(viewerSpace);
        for(const view of pose.views) {
          const depthInformation = isCpuOptimized ? frame.getDepthInformation(view)
                                                  : glBinding.getDepthInformation(view);

          if (state == TestStates.ShouldSucceedScheduleRAF
          || state == TestStates.ShouldSucceedTestDone) {
            t.step(() => {
              assert_not_equals(depthInformation, null);
            });
          } else {
            t.step(() => {
              assert_equals(depthInformation, null);
            });
          }
        }

        switch(state) {
          case TestStates.ShouldSucceedScheduleRAF:
            controller.clearDepthSensingData();
            state = TestStates.ShouldFailScheduleRAF;
            session.requestAnimationFrame(rafCb);
            break;
          case TestStates.ShouldFailScheduleRAF:
            controller.setDepthSensingData(DEPTH_SENSING_DATA);
            state = TestStates.ShouldSucceedTestDone;
            session.requestAnimationFrame(rafCb);
            break;
          case TestStates.ShouldSucceedTestDone:
            done = true;
            break;
        }
      };

      session.requestAnimationFrame(rafCb);

      return t.step_wait(() => done);
    });
  };
};
'use strict';

const TestStates = Object.freeze({
  "ShouldSucceedPauseScheduleRAF": 1,
  "ShouldFailResumeScheduleRAF": 2,
  "ShouldSucceedTestDone": 3,
});

const framesToWait = 10;
const pauseResumeTestFunction = function(session, controller, t, sessionObjects) {
  const isCpuOptimized = session.depthUsage === 'cpu-optimized';
  let state = TestStates.ShouldSucceedPauseScheduleRAF;

  return session.requestReferenceSpace('viewer').then((viewerSpace) => {
    let done = false;

    const glBinding = new XRWebGLBinding(session, sessionObjects.gl);

    let stepFrameCount = 0;
    let advanceState = false;

    const rafCb = function(time, frame) {
      const pose = frame.getViewerPose(viewerSpace);
      stepFrameCount++;
      for(const view of pose.views) {
        const depthInformation = isCpuOptimized ? frame.getDepthInformation(view)
                                                : glBinding.getDepthInformation(view);

        if (state == TestStates.ShouldSucceedPauseScheduleRAF
        || state == TestStates.ShouldSucceedTestDone) {
          t.step(() => {
            assert_true(session.depthActive);
          });
          // We have no guarantees about when data should start returning,
          // so we need to potentially wait a few frames.

          // Final chance. If we haven't advanced the state by now, fail the
          // test if it doesn't pass this time.
          if (stepFrameCount >= framesToWait) {
            t.step(() => {
              assert_not_equals(depthInformation, null);
            });
          }

          // Either we have data, or we've waited long enough to keep moving.
          if (depthInformation != null || stepFrameCount >= framesToWait) {
            advanceState = true;
          }
        } else {
          // Depth should stop being available immediately.
          t.step(() => {
            assert_false(session.depthActive);
            assert_equals(depthInformation, null);
          });
          advanceState = true;
        }
      }

      switch(state) {
        case TestStates.ShouldSucceedPauseScheduleRAF:
          if (advanceState) {
            session.pauseDepthSensing();
            for(const view of pose.views) {
              const newDepthInformation = isCpuOptimized ? frame.getDepthInformation(view)
                                                          : glBinding.getDepthInformation(view);
              t.step(()=> {
                // depthActive state should update and stop returning depth info
                // immediately.
                assert_false(session.depthActive);
                assert_equals(newDepthInformation, null);
              });
            }
            state = TestStates.ShouldFailResumeScheduleRAF;
            stepFrameCount = 0;
            advanceState = false;
          }
          session.requestAnimationFrame(rafCb);
          break;
        case TestStates.ShouldFailResumeScheduleRAF:
          if (advanceState) {
            session.resumeDepthSensing();
            // In pausing depth sensing, any controller data may have been
            // thrown away since the UA can "tear down" any depth controller.
            // So attempt to repopulate the data once we've resumed it.
            controller.setDepthSensingData(DEPTH_SENSING_DATA);
          t.step(()=> {
              // While depth data may not return for a few frames, depthActive
              // should be updated immediately.
              assert_true(session.depthActive);
            });
            state = TestStates.ShouldSucceedTestDone;
            stepFrameCount = 0;
            advanceState = false;
          }
          session.requestAnimationFrame(rafCb);
          break;
        case TestStates.ShouldSucceedTestDone:
          // If we are advancing the state, we can stop pumping the rAF, but
          // if we're still waiting for data to come back we need to keep it
          // going.
          if (advanceState) {
            done = true;
          } else {
            session.requestAnimationFrame(rafCb);
          }
          break;
      }
    };

    session.requestAnimationFrame(rafCb);

    return t.step_wait(() => done);
  });
};

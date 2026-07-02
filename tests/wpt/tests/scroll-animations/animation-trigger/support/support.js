
// Assert that the user agent under test supports AnimationTrigger.
// AnimationTrigger tests should do this sanity check before proceeding.
function assertAnimationTriggerSupport() {
  assert_true(document.documentElement.style.animationTrigger !== undefined);
}

const setScrollTop = (scroller, y) => {
  const scrollend_promise =
    waitForScrollEndFallbackToDelayWithoutScrollEvent(scroller);
  scroller.scrollTop = y;
  return scrollend_promise;
}

// TODO: replace "trigger" and "exit" with "activation" and "active"
function getRangeBoundariesForTest(trigger_start, trigger_end,
                                   exit_start, exit_end, scroller) {
  let rangeBoundaries = {
    scroller: scroller,
    offsetWithinTriggerRange: (trigger_start + trigger_end) / 2,
    offsetAboveTriggerRange: trigger_start - 10,
    offsetBelowTriggerRange: trigger_end + 10,
    offsetAboveExitRange: exit_start - 10,
    offsetBelowExitRange: exit_end + 10,
  };

  rangeBoundaries.enterTriggerRange = async () => {
    return setScrollTop(rangeBoundaries.scroller,
                        rangeBoundaries.offsetWithinTriggerRange);
  };
  rangeBoundaries.exitTriggerRangeAbove = async () => {
    return setScrollTop(rangeBoundaries.scroller,
                        rangeBoundaries.offsetAboveTriggerRange);
  };
  rangeBoundaries.exitTriggerRangeBelow = async () => {
    return setScrollTop(rangeBoundaries.scroller,
                        rangeBoundaries.offsetBelowTriggerRange);
  };
  rangeBoundaries.exitExitRangeAbove = async () => {
    return setScrollTop(rangeBoundaries.scroller,
                        rangeBoundaries.offsetAboveExitRange);
  };
  rangeBoundaries.exitExitRangeBelow = async () => {
    return setScrollTop(rangeBoundaries.scroller,
                        rangeBoundaries.offsetBelowExitRange);
  };

  return rangeBoundaries;
}

// A common pattern in timeline-trigger tests is to perform a scroll and verify
// that the trigger did (or didin't do) something. In these cases, we need one
// frame to let the trigger on the compositor thread observe the
// (main-thread-originating) scroll update, and one frame to let the main thread
// observe the trigger's response to the scroll update.
function runAndWaitForTriggerResponse(callback) {
  return runAndWaitForFrameUpdate(() => {
    callback();
  }).then(waitForNextFrame);
}

// Helper function for tests using timeline-trigger[1].
// This function scrolls into the activation range as configured by
// getRangeBoundariesForTest above.
//
// [1] https://drafts.csswg.org/css-animations-2/#timeline-triggers
const enter = (rangeBoundaries) => {
  return runAndWaitForTriggerResponse(() => {
    rangeBoundaries.enterTriggerRange();
  });
}

// Helper function for tests using timeline-trigger[1].
// This function scrolls outside the active range as configured by
// getRangeBoundariesForTest above.
//
// [1] https://drafts.csswg.org/css-animations-2/#timeline-triggers
const exit = (rangeBoundaries, exitAbove = true) => {
  return runAndWaitForTriggerResponse(() => {
    if (exitAbove) {
      rangeBoundaries.exitExitRangeAbove();
    } else {
      rangeBoundaries.exitExitRangeBelow();
    }
  });
}

// Helper function for animation-trigger tests. Aims to perform a scroll and
// observe the animation events indicated by |events_of_interest| and
// |events_should_fire|
async function testAnimationTrigger(test, scroll_fn, target,
                                    events_of_interest,  events_should_fire) {
  assertAnimationTriggerSupport();

  let evt_promises = [];
  for (let idx = 0; idx < events_of_interest.length; idx++) {
    const evt = events_of_interest[idx];
    const animationevent_promise = new Promise((resolve) => {
      const watcher_func = () => {
        if (!events_should_fire[idx]) {
          test.unreached_func(`received unexpected event: ${evt}.`)();
        }
        resolve();
      }

      target.addEventListener(evt, watcher_func,
        { once: true });

      // If we are not expecting the event, just wait for 3 frames before
      // continuing the test.
      if (!events_should_fire[idx]) {
        waitForAnimationFrames(3).then(() => {
          target.removeEventListener(evt, watcher_func);
          resolve();
        });
      }
    });

    evt_promises.push(animationevent_promise);
  }

  await scroll_fn();
  await Promise.all(evt_promises);
}

function computeContainOffset(scroller, subject, pct) {
  const contain_start = subject.offsetTop + subject.offsetHeight
    - scroller.offsetTop - scroller.clientHeight;
  const contain_end = subject.offsetTop - scroller.offsetTop;

  return contain_start + (pct / 100) * (contain_end - contain_start);
}

function setupAnimationAndTrigger(target, subject, duration,
                                  rangeStart = "contain 0%",
                                  rangeEnd = "contain 100%") {
  const animation = new Animation(
    new KeyframeEffect(
      target,
      [
        { transform: "scale(1)", backgroundColor: "yellow" },
        { transform: "scale(2)", backgroundColor: "yellow" },
      ],
      { duration: duration, fill: "both" }
    ));

  let trigger = new TimelineTrigger([{
    timeline: new ViewTimeline({ subject: subject, axis: "y" }),
    activationRangeStart: rangeStart,
    activationRangeEnd: rangeEnd
  }]);

  trigger.addAnimation(animation, "play-forwards", "play-backwards");
}

async function waitForAnimation(targetCurrentTime, animation) {
  return new Promise(resolve => {
    function waitForCurrentTime() {
      if ((targetCurrentTime - animation.currentTime) * animation.playbackRate <= 0) {
        resolve();
        return;
      }

      requestAnimationFrame(waitForCurrentTime);
    }
    waitForCurrentTime();
  });
}

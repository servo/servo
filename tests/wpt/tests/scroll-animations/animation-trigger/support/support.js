
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

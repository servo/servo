
// Used in trigger-scope-* tests to check whether an animation was triggered or
// not.
async function assert_playstate_and_current_time(target_id, animation, play_state) {
  // The animation might start on a different user-agent thread and need
  // a moment to get currentTime up to date.
  // TODO: This is incorrect use of waitForCompositorReady. We should remove it.
  await waitForCompositorReady();

  assert_equals(animation.playState, play_state,
    `animation on ${target_id} is ${play_state}.`);

  if (play_state === "running") {
    assert_greater_than(animation.currentTime, 0,
      `animation on ${target_id} has currentTime > 0.`);
  } else {
    assert_equals(animation.currentTime, 0,
      `animation on ${target_id} has currentTime == 0.`);
  }
}

const assert_element_animation_state = (element, play_state) => {
  assert_equals(element.getAnimations()[0].playState, play_state,
      `animation on ${element.id} has playState ${play_state}.`);
};

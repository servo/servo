// Methods used in trigger-scope-* tests to check whether an animation was
// triggered or not.

const assert_element_animation_state = (element, play_state) => {
  assert_equals(element.getAnimations()[0].playState, play_state,
      `animation on ${element.id} has playState ${play_state}.`);
};

const assert_elements_animation_state = (elements, play_state) => {
  for (const element of elements) {
    assert_element_animation_state(element, play_state);
  }
};

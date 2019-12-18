'use strict';

function assert_root_color_scheme(expected, description) {
  test(() => {
    assert_equals(getComputedStyle(document.documentElement).colorScheme, expected), "Check root element color scheme";
  }, description);
}

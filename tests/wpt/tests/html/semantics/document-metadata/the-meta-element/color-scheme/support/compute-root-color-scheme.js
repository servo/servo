'use strict';

function assert_root_color_scheme(expected_used_scheme, description) {
  function get_used_root_color_scheme() {
    let light = get_system_color("only light", "CanvasText");
    let dark = get_system_color("only dark", "CanvasText");
    assert_not_equals(light, dark, "CanvasText system color should be different with light and dark color schemes");
    let root = getComputedStyle(document.documentElement).color;
    assert_in_array(root, [light, dark], "Root color scheme should be either light or dark, or the text needs to be extended for newer color-schemes");
    return root == light ? "light" : "dark";
  }

  function get_system_color(scheme, color) {
    let div = document.createElement("div");
    div.style.color = color;
    div.style.colorScheme = scheme;

    document.documentElement.appendChild(div);
    let computed = getComputedStyle(div).color;
    div.remove();
    return computed;
  }

  test(() => {
    assert_equals(get_used_root_color_scheme(), expected_used_scheme);
    assert_equals(getComputedStyle(document.documentElement).colorScheme, "normal", "Root element's color-scheme should be 'normal'");
  }, description);
}

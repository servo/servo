function test_computed_style_aspect_ratio(tag, attributes, expected) {
  var elem = document.createElement(tag);
  for (name in attributes) {
    let val = attributes[name];
    if (val !== null)
      elem.setAttribute(name, val);
  }
  document.body.appendChild(elem);
  assert_equals(getComputedStyle(elem).aspectRatio, expected);
}

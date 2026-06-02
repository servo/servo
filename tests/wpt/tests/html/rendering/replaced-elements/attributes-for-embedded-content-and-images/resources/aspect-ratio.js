function test_computed_style_aspect_ratio(tag, attributes, expected) {
  test(function() {
    var elem = document.createElement(tag);
    for (name in attributes) {
      let val = attributes[name];
      if (val !== null)
        elem.setAttribute(name, val);
    }
    document.body.appendChild(elem);
    let aspectRatio = getComputedStyle(elem).aspectRatio;
    assert_equals(aspectRatio, expected);
    elem.remove();
  }, `Computed style test: ${tag} with ${JSON.stringify(attributes)}`);
}

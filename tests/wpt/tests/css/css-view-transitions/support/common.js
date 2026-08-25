function assert_animations(filterFn, mappingFn, expected, message) {
  const values = document.getAnimations()
        .filter(filterFn)
        .map(mappingFn);
  const unique_values = [...new Set(values)].sort();

  const format = entries => entries.join(", ");
  assert_equals(format(unique_values), format(expected), message);
}

function assert_animation_pseudos(element, expected, message) {
  const filterFn = a => a.effect.target == element;
  const mappingFn = a => a.effect.pseudoElement;
  return assert_animations(filterFn, mappingFn, expected, message);
}

function assert_animation_names(target, expected, message) {
  const filterFn = a => a.effect.pseudoElement == target;
  const mappingFn = a => a.animationName;
  return assert_animations(filterFn, mappingFn, expected, message);
}

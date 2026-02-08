
// Helper function to assert that the correct scroll-marker among those in the
// provided list is selected.
function verifySelectedMarker(selected_idx, items, selected_color,
                              unselected_color) {
  for (let idx = items.length - 1; idx >= 0; --idx) {
    const should_be_selected = idx == selected_idx;
    let expected_color = should_be_selected ? selected_color : unselected_color;
    const color =
      getComputedStyle(items[idx], "::scroll-marker").backgroundColor;
    assert_equals(color, expected_color,
      `marker ${idx} should be ${should_be_selected ? "" : "un"}selected.`);
  }
}

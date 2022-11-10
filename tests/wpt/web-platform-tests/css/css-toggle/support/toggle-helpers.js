async function wait_for_toggle_creation(element) {
  // TODO(crbug.com/1250716): The spec is vague about when toggles need to be
  // created, and whether :toggle() pseudo-classes will update within the same
  // update.  See https://github.com/tabatkins/css-toggle/issues/27 .  For
  // now, we call elementFromPoint (which in Chromium flushes to PrePaint
  // clean), which isn't a long term solution!
  document.elementFromPoint(10, 10);
}

async function set_up_single_toggle_in(container, toggle_style) {
  let div = document.createElement("div");
  div.style.toggle = toggle_style;
  container.replaceChildren(div);
  await wait_for_toggle_creation(div);
  return div;
}

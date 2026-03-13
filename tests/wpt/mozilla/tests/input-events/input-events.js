// Get the bounding client rect of the given node, but give it a bit
// of padding to account for borders and padding added by the text input.
function getTextEntryContentRect(node, padding = 8) {
  let rectangle = node.getBoundingClientRect();
  rectangle.x += padding;
  rectangle.y += padding;
  rectangle.width -= padding * 2;
  rectangle.height -= padding * 2;
  return rectangle;
}

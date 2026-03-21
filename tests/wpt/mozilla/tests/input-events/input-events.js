// Get the content rectangle of the given text entry node, accounting for an
// optional scale and 90 degree rotation transform.
function getTextEntryContentRect(node, scale = 1, rotate90Degrees = false) {
  let cssStyleTextToPixels = (value) => {
    return parseFloat(value) || 0;
  };

  let computedStyle = getComputedStyle(node);
  let leftInset = cssStyleTextToPixels(computedStyle.borderLeftWidth) +
    cssStyleTextToPixels(computedStyle.paddingLeft) * scale;
  let topInset = cssStyleTextToPixels(computedStyle.borderTopWidth) +
    cssStyleTextToPixels(computedStyle.paddingTop) * scale;
  let rightInset = cssStyleTextToPixels(computedStyle.borderRightWidth) +
    cssStyleTextToPixels(computedStyle.paddingRight) * scale;
  let bottomInset = cssStyleTextToPixels(computedStyle.borderBottomWidth) +
    cssStyleTextToPixels(computedStyle.paddingBottom) * scale;

  if (rotate90Degrees) {
    [leftInset, topInset, rightInset, bottomInset] = [bottomInset, leftInset, topInset, rightInset];
  }

  let rectangle = node.getBoundingClientRect()
  rectangle.x += leftInset;
  rectangle.y += topInset;
  rectangle.width -= (leftInset + rightInset);
  rectangle.height -= (topInset + bottomInset);
  return rectangle;
}

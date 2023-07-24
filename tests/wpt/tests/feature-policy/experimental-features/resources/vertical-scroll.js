function rectMaxY(rect) {
  return rect.height + rect.y;
}

function rectMaxX(rect) {
  return rect.width + rect.x;
}

function isEmptyRect(rect) {
  return !rect.width || !rect.height;
}

// Returns true if the given rectangles intersect.
function rects_intersect(rect1, rect2) {
  if (isEmptyRect(rect1) || isEmptyRect(rect2))
    return false;
  return rect1.x < rectMaxX(rect2) &&
         rect2.x < rectMaxX(rect1) &&
         rect1.y < rectMaxY(rect2) &&
         rect2.y < rectMaxY(rect1);
}

function rectToString(rect) {
  return `Location: (${rect.x}, ${rect.y}) Size: (${rect.width}, ${rect.height})`;
}

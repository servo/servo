'use strict';

/**
 * Returns an array of advances for all characters in the descendants
 * of the specified element.
 *
 * Technically speaking, advances and glyph bounding boxes are different things,
 * and advances are not exposed. This function computes approximate values from
 * bounding boxes.
 */
function getCharAdvances(element) {
  const style = getComputedStyle(element);
  const is_vertical = style.writingMode.startsWith('vertical');
  const range = document.createRange();
  const all_bounds = []

  function walk(element) {
    for (const node of element.childNodes) {
      const nodeType = node.nodeType;
      if (nodeType === Node.TEXT_NODE) {
        const text = node.nodeValue;
        for (let i = 0; i < text.length; ++i) {
          range.setStart(node, i);
          range.setEnd(node, i + 1);
          let bounds = range.getBoundingClientRect();
          // Transpose if it's in vertical flow. Guarantee that top < bottom
          // and left < right are always true.
          if (is_vertical) {
            bounds = {
              left: bounds.top,
              top: bounds.left,
              right: bounds.bottom,
              bottom: bounds.right
            };
          }
          all_bounds.push(bounds);
        }
      } else if (nodeType === Node.ELEMENT_NODE) {
        walk(node);
      }
    }
  }
  walk(element);
  all_bounds.sort(function(bound_a, bound_b) {
    if (bound_a.bottom <= bound_b.top) {
      return -1;
    }
    if (bound_b.bottom <= bound_a.top) {
      return 1;
    }
    return bound_a.left - bound_b.left;
  });
  let origin = undefined;
  let blockEnd = -1;
  const advances = [];
  for (let bounds of all_bounds) {
    // Check if this is on the same line.
    if (bounds.top >= blockEnd) {
      origin = undefined;
      blockEnd = bounds.bottom;
    }
    // For the first character, the left bound is closest to the origin.
    if (origin === undefined)
      origin = bounds.left;
    // The right bound is a good approximation of the next origin.
    advances.push(bounds.right - origin);
    origin = bounds.right;
  }
  return advances;
}

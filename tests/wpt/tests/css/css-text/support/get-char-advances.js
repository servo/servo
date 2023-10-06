'use strict';

/**
 * Returns an array of advances (widths) for all characters in the descendants
 * of the specified element.
 */
function getCharAdvances(element) {
  const range = document.createRange();
  let advances = [];
  for (const node of element.childNodes) {
    const nodeType = node.nodeType;
    if (nodeType === Node.TEXT_NODE) {
      const text = node.nodeValue;
      for (let i = 0; i < text.length; ++i) {
        range.setStart(node, i);
        range.setEnd(node, i + 1);
        const bounds = range.getBoundingClientRect();
        advances.push(bounds.width);
      }
    } else if (nodeType === Node.ELEMENT_NODE) {
      advances = advances.concat(getCharAdvances(node));
    }
  }
  return advances;
}

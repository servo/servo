"use strict";

/**
 * Find characters in a container and creates CSS Highlights for them.
 * @param {Node} container - The container node to search within.
 */
function highlightChars(container) {
  const chars = {
    "「": "open",
    "」": "close",
    "（": "open",
    "）": "close",
    "。": "dot",
    "、": "dot",
    "．": "dot",
    "，": "dot",
    "：": "colon",
    "；": "colon",
  };
  const style = [
    "::highlight(open) { background-color: orange; }",
    "::highlight(close) { background-color: springgreen; }",
    "::highlight(dot) { background-color: skyblue; }",
    "::highlight(colon) { background-color: wheat; }",
  ].join("\n");
  const style_element = document.createElement("style");
  style_element.textContent = style;
  document.head.appendChild(style_element);

  const walker = document.createTreeWalker(container, NodeFilter.SHOW_TEXT);
  while (walker.nextNode()) {
    const textNode = walker.currentNode;
    const text = textNode.nodeValue;
    for (let i = 0; i < text.length; ++i) {
      const char = text[i];
      const name = chars[char];
      if (!name) {
        continue;
      }
      let highlight = CSS.highlights.get(name);
      if (!highlight) {
        highlight = new Highlight();
        CSS.highlights.set(name, highlight);
      }
      const range = document.createRange();
      range.setStart(textNode, i);
      range.setEnd(textNode, i + 1);
      highlight.add(range);
    }
  }
}

window.addEventListener("load", () => {
  const container = document.getElementById("container");
  highlightChars(container ? container : document.body);
});

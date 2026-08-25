class SelectionTestUtils {
  static getSelectionRangeArray(selection) {
    const ranges = [];
    for (let i = 0; i < selection.rangeCount; i++) {
      ranges.push(selection.getRangeAt(i));
    }
    return ranges;
  }

  static getRangeArrayDescription(arrayOfRanges) {
    if (arrayOfRanges === null) {
      return "null";
    }
    if (arrayOfRanges === undefined) {
      return "undefined";
    }
    if (!Array.isArray(arrayOfRanges)) {
      return "Unknown Object";
    }
    if (arrayOfRanges.length === 0) {
      return "[]";
    }
    let result = "";
    for (let range of arrayOfRanges) {
      if (result === "") {
        result = "[";
      } else {
        result += ",";
      }
      result += `{${SelectionTestUtils.getRangeDescription(range)}}`;
    }
    result += "]";
    return result;
  }

  static getNodeDescription(node) {
    if (!node) {
      return "null";
    }
    switch (node.nodeType) {
      case Node.TEXT_NODE:
      case Node.COMMENT_NODE:
      case Node.CDATA_SECTION_NODE:
        return `${node.nodeName} "${node.data.replaceAll("\n", "\\\\n")}"`;
      case Node.ELEMENT_NODE:
        return `<${node.nodeName.toLowerCase()}${
            node.hasAttribute("id") ? ` id="${node.getAttribute("id")}"` : ""
          }${
            node.hasAttribute("class") ? ` class="${node.getAttribute("class")}"` : ""
          }${
            node.hasAttribute("contenteditable")
              ? ` contenteditable="${node.getAttribute("contenteditable")}"`
              : ""
          }${
            node.inert ? ` inert` : ""
          }${
            node.hidden ? ` hidden` : ""
          }${
            node.readonly ? ` readonly` : ""
          }${
            node.disabled ? ` disabled` : ""
          }>`;
      default:
        return `${node.nodeName}`;
    }
  }

  static getRangeDescription(range) {
    if (range === null) {
      return "null";
    }
    if (range === undefined) {
      return "undefined";
    }
    return range.startContainer == range.endContainer &&
      range.startOffset == range.endOffset
      ? `(${SelectionTestUtils.getNodeDescription(range.startContainer)}, ${range.startOffset})`
      : `(${SelectionTestUtils.getNodeDescription(range.startContainer)}, ${
          range.startOffset
        }) - (${SelectionTestUtils.getNodeDescription(range.endContainer)}, ${range.endOffset})`;
  }
}

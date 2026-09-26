// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://drafts.csswg.org/geometry/#DOMPoint
// https://drafts.csswg.org/geometry/#DOMRect
// https://drafts.csswg.org/geometry/#DOMQuad
// https://drafts.csswg.org/geometry/#DOMMatrix

"use strict";

idl_test(
  ["geometry"],
  [],
  idlArray => {
    const domRectListList = [];
    if ("document" in self) {
      domRectListList.push(document.getElementById('log').getClientRects());
    }
    idlArray.add_objects({
      DOMPointReadOnly: ["new DOMPointReadOnly()"],
      DOMPoint: ["new DOMPoint()"],
      DOMRectReadOnly: ["new DOMRectReadOnly()"],
      DOMRect: ["new DOMRect()"],
      DOMRectList: domRectListList,
      DOMQuad: ["new DOMQuad()"],
      DOMMatrixReadOnly: ["new DOMMatrixReadOnly()", "DOMMatrixReadOnly.fromMatrix({is2D: false})"],
      DOMMatrix: ["new DOMMatrix()", "DOMMatrix.fromMatrix({is2D: false})"]
    });
    idlArray.prevent_multiple_testing("DOMMatrixReadOnly");
    idlArray.prevent_multiple_testing("DOMMatrix");
  }
);

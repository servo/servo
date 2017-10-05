"use strict";

var idlArray = new IdlArray();

function doTest(idl) {
  idlArray.add_idls(idl);
  idlArray.add_objects({
    DOMPointReadOnly: ["new DOMPointReadOnly()"],
    DOMPoint: ["new DOMPoint()"],
    DOMRectReadOnly: ["new DOMRectReadOnly()"],
    DOMRect: ["new DOMRect()"],
    DOMQuad: ["new DOMQuad()"],
    DOMMatrixReadOnly: ["new DOMMatrixReadOnly()", "DOMMatrixReadOnly.fromMatrix({is2D: false})"],
    DOMMatrix: ["new DOMMatrix()", "DOMMatrix.fromMatrix({is2D: false})"],
  });
  idlArray.test();
  done();
}

promise_test(function() {
  return fetch("/interfaces/geometry.idl").then(response => response.text())
                                          .then(doTest);
}, "Test driver");

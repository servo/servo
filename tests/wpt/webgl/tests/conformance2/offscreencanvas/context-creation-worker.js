/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

importScripts("../../js/tests/canvas-tests-utils.js");
self.onmessage = function(e) {
    if (contextCreation('webgl2'))
      self.postMessage("Test passed");
    else
      self.postMessage("Test failed");
};

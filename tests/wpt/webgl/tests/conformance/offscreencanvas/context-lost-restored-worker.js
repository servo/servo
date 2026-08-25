/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

importScripts("../../js/tests/canvas-tests-utils.js");
self.onmessage = function(e) {
    if (!setupTest())
        self.postMessage("Test failed");

    canvas.addEventListener("webglcontextlost", function(e) {
        if (!testLostContext(e))
            self.postMessage("Test failed");
        // restore the context after this event has exited.
        setTimeout(function() {
            // we didn't call prevent default so we should not be able to restore the context
            if (!compareGLError(gl.INVALID_OPERATION, "WEBGL_lose_context.restoreContext()"))
                self.postMessage("Test failed");
            testLosingAndRestoringContext().then(function() {
                self.postMessage("Test passed");
            }, function() {
                self.postMessage("Test failed");
            });
        }, 0);
    });
    canvas.addEventListener("webglcontextrestored", function() {
        self.postMessage("Test failed");
    });
    allowRestore = false;
    contextLostEventFired = false;
    contextRestoredEventFired = false;

    if (!testOriginalContext())
        self.postMessage("Test failed");
    WEBGL_lose_context.loseContext();
    // The context should be lost immediately.
    if (!gl.isContextLost())
        self.postMessage("Test failed");
    if (gl.getError() != gl.CONTEXT_LOST_WEBGL)
        self.postMessage("Test failed");
    if (gl.getError() != gl.NO_ERROR)
        self.postMessage("Test failed");
    // gl methods should be no-ops
    if (!compareGLError(gl.NO_ERROR, "gl.blendFunc(gl.TEXTURE_2D, gl.TEXTURE_CUBE_MAP)"))
        self.postMessage("Test failed");
    // but the event should not have been fired.
    if (contextLostEventFired)
        self.postMessage("Test failed");
}


/*
** Copyright (c) 2016 The Khronos Group Inc.
**
** Permission is hereby granted, free of charge, to any person obtaining a
** copy of this software and/or associated documentation files (the
** "Materials"), to deal in the Materials without restriction, including
** without limitation the rights to use, copy, modify, merge, publish,
** distribute, sublicense, and/or sell copies of the Materials, and to
** permit persons to whom the Materials are furnished to do so, subject to
** the following conditions:
**
** The above copyright notice and this permission notice shall be included
** in all copies or substantial portions of the Materials.
**
** THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
** EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
** MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
** IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
** CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
** TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
** MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
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


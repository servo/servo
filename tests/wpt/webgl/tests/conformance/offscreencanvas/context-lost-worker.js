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
    canvas = new OffscreenCanvas(10, 10);
    gl = canvas.getContext('webgl');

    // call testValidContext() before checking for the extension, because this is where we check
    // for the isContextLost() method, which we want to do regardless of the extension's presence.
    if (!testValidContext())
        self.postMessage("Test failed");

    extension = gl.getExtension("WEBGL_lose_context");
    // need an extension that exposes new API methods.
    OES_vertex_array_object = gl.getExtension("OES_vertex_array_object");
    if (extension == null || OES_vertex_array_object == null)
        self.postMessage("Test failed");

    // We need to initialize |uniformLocation| before losing context.
    // Otherwise gl.getUniform() when context is lost will throw.
    uniformLocation = gl.getUniformLocation(program, "tex");
    extension.loseContext();

    canvas.addEventListener("webglcontextlost", function() {
        if (testLostContextWithoutRestore())
            self.postMessage("Test passed");
        else
            self.postMessage("Test failed");
    }, false);
}


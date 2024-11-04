/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

importScripts("../../js/tests/canvas-tests-utils.js");
self.onmessage = function(e) {
    canvas = new OffscreenCanvas(10, 10);
    gl = canvas.getContext('webgl');

    // call testValidContext() before checking for the extension, because this is where we check
    // for the isContextLost() method, which we want to do regardless of the extension's presence.
    self.postMessage({fail: !testValidContext(), msg: "testValidContext()"});

    WEBGL_lose_context = gl.getExtension("WEBGL_lose_context");
    self.postMessage({fail: !WEBGL_lose_context, msg: "WEBGL_lose_context"});

    // need an extension that exposes new API methods.
    OES_vertex_array_object = gl.getExtension("OES_vertex_array_object");
    self.postMessage({fail: !OES_vertex_array_object, msg: "OES_vertex_array_object"});

    // We need to initialize |uniformLocation| before losing context.
    // Otherwise gl.getUniform() when context is lost will throw.
    uniformLocation = gl.getUniformLocation(program, "tex");
    WEBGL_lose_context.loseContext();

    canvas.addEventListener("webglcontextlost", function() {
        self.postMessage({fail: !testLostContextWithoutRestore(), msg: "testLostContextWithoutRestore()",
            finishTest:true});
    }, false);
}


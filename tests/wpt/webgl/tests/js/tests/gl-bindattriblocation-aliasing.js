/*
** Copyright (c) 2018 The Khronos Group Inc.
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

"use strict";

var runBindAttribLocationAliasingTest = function(wtu, gl, glFragmentShader, vertexShaderTemplate) {
    var typeInfo = [
        { type: 'float',    asVec4: 'vec4(0.0, $(var), 0.0, 1.0)' },
        { type: 'vec2',     asVec4: 'vec4($(var), 0.0, 1.0)' },
        { type: 'vec3',     asVec4: 'vec4($(var), 1.0)' },
        { type: 'vec4',     asVec4: '$(var)' },
    ];
    var maxAttributes = gl.getParameter(gl.MAX_VERTEX_ATTRIBS);
    // Test all type combinations of a_1 and a_2.
    typeInfo.forEach(function(typeInfo1) {
        typeInfo.forEach(function(typeInfo2) {
            debug('attribute_1: ' + typeInfo1.type + ' attribute_2: ' + typeInfo2.type);
            var replaceParams = {
                type_1: typeInfo1.type,
                type_2: typeInfo2.type,
                gl_Position_1: wtu.replaceParams(typeInfo1.asVec4, {var: 'a_1'}),
                gl_Position_2: wtu.replaceParams(typeInfo2.asVec4, {var: 'a_2'})
            };
            var strVertexShader = wtu.replaceParams(vertexShaderTemplate, replaceParams);
            var glVertexShader = wtu.loadShader(gl, strVertexShader, gl.VERTEX_SHADER);
            assertMsg(glVertexShader != null, "Vertex shader compiled successfully.");
            // Bind both a_1 and a_2 to the same position and verify the link fails.
            // Do so for all valid positions available.
            for (var l = 0; l < maxAttributes; l++) {
                var glProgram = gl.createProgram();
                gl.bindAttribLocation(glProgram, l, 'a_1');
                gl.bindAttribLocation(glProgram, l, 'a_2');
                gl.attachShader(glProgram, glVertexShader);
                gl.attachShader(glProgram, glFragmentShader);
                gl.linkProgram(glProgram);
                var linkStatus = gl.getProgramParameter(glProgram, gl.LINK_STATUS);
                assertMsg(!linkStatus, "Link should fail when both attributes are aliased to location " + l);
            }
        });
    });
};
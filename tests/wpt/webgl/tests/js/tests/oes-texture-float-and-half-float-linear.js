/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

function testTexLinear(gl, extensionName, internalFormatWebgl2, pixelType) {
    var wtu = WebGLTestUtils;

    // Before the extension is enabled
    var extensionEnabled = false;
    runTestSuite(extensionEnabled);

    if (!gl.getExtension(extensionName))
        testPassed("No " + extensionName + " support -- this is legal");
    else {
        // After the extension is enabled
        extensionEnabled = true;
        runTestSuite(extensionEnabled);
    }

    function runTestSuite(extensionEnabled)
    {
        var magF = [gl.NEAREST, gl.LINEAR];
        var minF = [gl.NEAREST, gl.LINEAR, gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST_MIPMAP_LINEAR, gl.LINEAR_MIPMAP_NEAREST, gl.LINEAR_MIPMAP_LINEAR];
        var tex2DFShader = [
            'uniform sampler2D tex;',
            'void main() {',
            '    gl_FragData[0] = texture2D(tex, vec2(0.5, 0.5)) * vec4(4.0, 2.0, 2.0, 1);',
            '}'].join('\n');

        var positionVertexShader = [
           'attribute vec4 vPosition;',
           'void main() {',
           '    gl_Position = vPosition;',
           '}'].join('\n');

        var texCubeFShader = [
            'uniform samplerCube tex;',
            'void main() {',
            '    gl_FragColor = textureCube(tex, normalize(vec3(0.5, 0.5, 1))) * vec4(4.0, 2.0, 2.0, 1);',
            '}'].join('\n');

        var vs = wtu.loadShader(gl, positionVertexShader, gl.VERTEX_SHADER);
        var fs_2d = wtu.loadShader(gl, tex2DFShader, gl.FRAGMENT_SHADER);
        var fs_cube = wtu.loadShader(gl, texCubeFShader, gl.FRAGMENT_SHADER);

        // TEXTURE_2D
        var program = wtu.setupProgram(gl, [vs, fs_2d]);
        gl.useProgram(program);
        wtu.setupUnitQuad(gl);
        for (var kk = 0; kk < 2; ++kk) {
            for (var ii = 0; ii < 6; ++ii) {
                var linear = false;
                if (magF[kk] == gl.LINEAR || (minF[ii] != gl.NEAREST && minF[ii] != gl.NEAREST_MIPMAP_NEAREST))
                    linear = true;
                var color = (!extensionEnabled && linear) ? [0, 0, 0, 255] : [255, 255, 255, 255];
                runEachTest(gl.TEXTURE_2D, magF[kk], minF[ii], linear, extensionEnabled, color);
            }
        }

        // TEXTURE_CUBE_MAP
        var programCube = wtu.setupProgram(gl, [vs, fs_cube]);
        gl.useProgram(programCube);
        wtu.setupUnitQuad(gl);
        for (var kk = 0; kk < 2; ++kk) {
            for (var ii = 0; ii < 6; ++ii) {
                var linear = false;
                if (magF[kk] == gl.LINEAR || (minF[ii] != gl.NEAREST && minF[ii] != gl.NEAREST_MIPMAP_NEAREST))
                    linear = true;
                var color = (!extensionEnabled && linear) ? [0, 0, 0, 255] : [255, 255, 255, 255];
                runEachTest(gl.TEXTURE_CUBE_MAP, magF[kk], minF[ii], linear, extensionEnabled, color);
            }
        }
    }

    function runEachTest(textureTarget, magFilter, minFilter, linear, extensionEnabled, expected)
    {
        const format = gl.RGBA;
        let internalFormat = format;
        if (wtu.isWebGL2(gl)) {
            internalFormat = gl[internalFormatWebgl2];
        }
        var numberOfChannels = 4;
        debug("");
        debug("testing target: " + wtu.glEnumToString(gl,textureTarget) +
            ", testing format: " + wtu.glEnumToString(gl, format) +
            ", magFilter is: " + wtu.glEnumToString(gl, magFilter) +
            ", minFilter is: " + wtu.glEnumToString(gl, minFilter) +
            ", " + extensionName + " is " +  (extensionEnabled ? "enabled": "not enabled")
            );

        // Generate data.
        var width = 4;
        var height = 4;
        var canvas2d = document.createElement('canvas');
        canvas2d.width = width;
        canvas2d.height = height;
        var ctx2d = canvas2d.getContext('2d');
        var color = [64, 128, 128, 255];
        ctx2d.fillStyle = "rgba(" + color[0] + "," + color[1] + "," + color[2] + "," + color[3] + ")";
        ctx2d.fillRect(0, 0, width, height);

        var texture = gl.createTexture();
        gl.bindTexture(textureTarget, texture);
        gl.texParameteri(textureTarget, gl.TEXTURE_MAG_FILTER, magFilter);
        gl.texParameteri(textureTarget, gl.TEXTURE_MIN_FILTER, minFilter);
        gl.texParameteri(textureTarget, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(textureTarget, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

        if (textureTarget == gl.TEXTURE_2D) {
            gl.texImage2D(gl.TEXTURE_2D, 0, internalFormat, format, gl[pixelType], canvas2d);
            if (minFilter != gl.NEAREST && minFilter != gl.LINEAR) {
                wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors during texture setup");
                gl.generateMipmap(gl.TEXTURE_2D);
                if (gl.getError() != gl.NO_ERROR) {
                    debug("generateMipmap failed for floating-point TEXTURE_2D -- this is legal -- skipping the rest of this test");
                    return;
                }
            }
        } else if (textureTarget == gl.TEXTURE_CUBE_MAP) {
            var targets = [
                gl.TEXTURE_CUBE_MAP_POSITIVE_X,
                gl.TEXTURE_CUBE_MAP_NEGATIVE_X,
                gl.TEXTURE_CUBE_MAP_POSITIVE_Y,
                gl.TEXTURE_CUBE_MAP_NEGATIVE_Y,
                gl.TEXTURE_CUBE_MAP_POSITIVE_Z,
                gl.TEXTURE_CUBE_MAP_NEGATIVE_Z];
                for (var tt = 0; tt < targets.length; ++tt)
                    gl.texImage2D(targets[tt], 0, internalFormat, format, gl[pixelType], canvas2d);
                if (minFilter != gl.NEAREST && minFilter != gl.LINEAR) {
                    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors during texture setup");
                    gl.generateMipmap(gl.TEXTURE_CUBE_MAP);
                    if (gl.getError() != gl.NO_ERROR) {
                        debug("generateMipmap failed for floating-point TEXTURE_CUBE_MAP -- this is legal -- skipping the rest of this test");
                        return;
                    }
                }
        }
        wtu.clearAndDrawUnitQuad(gl);
        if (!linear) {
            wtu.glErrorShouldBe(gl, gl.NO_ERROR, pixelType + " texture with non-Linear filter should succeed with NO_ERROR no matter whether " + extensionName + " is enabled or not");
        } else if (!extensionEnabled) {
            wtu.glErrorShouldBe(gl, gl.NO_ERROR, pixelType + " texture with Linear filter should produce [0, 0, 0, 1.0] with NO_ERROR if " + extensionName + " isn't enabled");
        } else {
            wtu.glErrorShouldBe(gl, gl.NO_ERROR, pixelType + " texture with Linear filter should succeed with NO_ERROR if " + extensionName + " is enabled");
        }

        wtu.checkCanvas(gl, expected, "should be " + expected[0] + "," + expected[1]  + "," +  expected[2] + "," + expected[3]);
    }
}

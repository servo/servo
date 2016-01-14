/*
** Copyright (c) 2014 The Khronos Group Inc.
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

function generateTest(pixelFormat, pixelType, prologue) {
    var wtu = WebGLTestUtils;
    var gl = null;
    var successfullyParsed = false;

    var init = function()
    {
        description('Verify texImage2D and texSubImage2D code paths taking webgl canvas elements (' + pixelFormat + '/' + pixelType + ')');

        gl = wtu.create3DContext("example");

        if (!prologue(gl)) {
            finishTest();
            return;
        }

        var program = wtu.setupTexturedQuad(gl);

        gl.clearColor(0,0,0,1);
        gl.clearDepth(1);

        runTest();
    }

    function setCanvasToRedGreen(ctx) {
      var width = ctx.canvas.width;
      var height = ctx.canvas.height;
      var halfHeight = Math.floor(height / 2);

      ctx.viewport(0, 0, width, height);

      ctx.enable(ctx.SCISSOR_TEST);
      ctx.scissor(0, 0, width, halfHeight);
      ctx.clearColor(1.0, 0, 0, 1.0);
      ctx.clear(ctx.COLOR_BUFFER_BIT);
      ctx.scissor(0, halfHeight, width, height - halfHeight);
      ctx.clearColor(0.0, 1.0, 0, 1.0);
      ctx.clear(ctx.COLOR_BUFFER_BIT);
      ctx.disable(ctx.SCISSOR_TEST);
    }

    function setCanvasTo257x257(ctx) {
      ctx.canvas.width = 257;
      ctx.canvas.height = 257;
      setCanvasToRedGreen(ctx);
    }

    function setCanvasTo1x2(ctx) {
      ctx.canvas.width = 1;
      ctx.canvas.height = 2;
      setCanvasToRedGreen(ctx);
    }

    function runOneIteration(canvas, useTexSubImage2D, flipY, opt_texture)
    {
        debug('Testing ' + (useTexSubImage2D ? 'texSubImage2D' : 'texImage2D') +
              ' with flipY=' + flipY + ' canvas size: ' + canvas.width + 'x' + canvas.height +
              ' with red-green');
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        if (!opt_texture) {
            var texture = gl.createTexture();
            // Bind the texture to texture unit 0
            gl.bindTexture(gl.TEXTURE_2D, texture);
            // Set up texture parameters
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        } else {
            var texture = opt_texture;
        }
        // Set up pixel store parameters
        gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, flipY);
        gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, false);
        gl.pixelStorei(gl.UNPACK_COLORSPACE_CONVERSION_WEBGL, gl.NONE);
        // Upload the image into the texture
        if (useTexSubImage2D) {
            // Initialize the texture to black first
            gl.texImage2D(gl.TEXTURE_2D, 0, gl[pixelFormat], canvas.width, canvas.height, 0,
                          gl[pixelFormat], gl[pixelType], null);
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, gl[pixelFormat], gl[pixelType], canvas);
        } else {
            gl.texImage2D(gl.TEXTURE_2D, 0, gl[pixelFormat], gl[pixelFormat], gl[pixelType], canvas);
        }

        // Draw the triangles
        wtu.clearAndDrawUnitQuad(gl, [0, 255, 0, 255]);

        var width = gl.canvas.width;
        var height = gl.canvas.height;
        var halfHeight = Math.floor(height / 2);
        var top = flipY ? (height - halfHeight) : 0;
        var bottom = flipY ? 0 : (height - halfHeight);

        // Check the top and bottom halves and make sure they have the right color.
        var red = [255, 0, 0];
        var green = [0, 255, 0];
        debug("Checking " + (flipY ? "top" : "bottom"));
        wtu.checkCanvasRect(gl, 0, bottom, width, halfHeight, red,
                            "shouldBe " + red);
        debug("Checking " + (flipY ? "bottom" : "top"));
        wtu.checkCanvasRect(gl, 0, top, width, halfHeight, green,
                            "shouldBe " + green);

        if (false) {
          var ma = wtu.makeImageFromCanvas(canvas);
          document.getElementById("console").appendChild(ma);

          var m = wtu.makeImageFromCanvas(gl.canvas);
          document.getElementById("console").appendChild(m);
          document.getElementById("console").appendChild(document.createElement("hr"));
        }

        return texture;
    }

    function runTest()
    {
        var ctx = wtu.create3DContext();
        var canvas = ctx.canvas;

        var count = 4;
        var caseNdx = 0;

        var cases = [
            { sub: false, flipY: true, init: setCanvasTo1x2 },
            { sub: false, flipY: false },
            { sub: true,  flipY: true },
            { sub: true,  flipY: false },
            { sub: false, flipY: true, init: setCanvasTo257x257 },
            { sub: false, flipY: false },
            { sub: true,  flipY: true },
            { sub: true,  flipY: false },
        ];

        var texture;
        function runNextTest() {
            var c = cases[caseNdx];
            if (c.init) {
              c.init(ctx);
            }
            texture = runOneIteration(canvas, c.sub, c.flipY, texture);
            // for the first 2 iterations always make a new texture.
            if (count > 2) {
              texture = undefined;
            }
            ++caseNdx;
            if (caseNdx == cases.length) {
                caseNdx = 0;
                --count;
                if (!count) {
                    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
                    finishTest();
                    return;
                }
            }
            wtu.waitForComposite(runNextTest);
        }
        runNextTest();
    }

    return init;
}

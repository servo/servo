/*
** Copyright (c) 2017 The Khronos Group Inc.
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

// This block needs to be outside the onload handler in order for this
// test to run reliably in WebKit's test harness (at least the
// Chromium port). https://bugs.webkit.org/show_bug.cgi?id=87448
initTestingHarness();

var old = debug;
var debug = function(msg) {
  bufferedLogToConsole(msg);
  old(msg);
};

function generateTest(desc,
                      internalFormat, pixelFormat, pixelType, prologue, resourcePath, defaultContextVersion,
                      videos) {
    var wtu = WebGLTestUtils;
    var tiu = TexImageUtils;
    var gl = null;
    var c2d = null;
    var successfullyParsed = false;
    var redColor = [255, 0, 0];
    var greenColor = [0, 255, 0];
    var currentTolerance = 0;

    function init()
    {
        description(desc + ' (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

        // Set the default context version while still allowing the webglVersion URL query string to override it.
        wtu.setDefault3DContextVersion(defaultContextVersion);
        gl = wtu.create3DContext("example");

        // Subsume 2D canvas tests into this test case since they usually go down similar code paths and
        // these tests are usually already set up to run with hardware accelerated video decoding.
        c2d = document.getElementById("c2d").getContext("2d");

        if (!prologue(gl)) {
            finishTest();
            return;
        }

        switch (gl[pixelFormat]) {
          case gl.RED:
          case gl.RED_INTEGER:
            greenColor = [0, 0, 0];
            break;
          default:
            break;
        }

        gl.clearColor(0,0,0,1);
        gl.clearDepth(1);

        runAllTests();
    }

    function runOneIteration(videoElement, useTexSubImage2D, flipY, topColor, bottomColor, sourceSubRectangle, program, bindingTarget)
    {
        sourceSubRectangleString = '';
        if (sourceSubRectangle) {
            sourceSubRectangleString = ' sourceSubRectangle=' + sourceSubRectangle;
        }
        debug('Testing ' + (useTexSubImage2D ? 'texSubImage2D' : 'texImage2D') +
              ' with flipY=' + flipY + ' bindingTarget=' +
              (bindingTarget == gl.TEXTURE_2D ? 'TEXTURE_2D' : 'TEXTURE_CUBE_MAP') +
              sourceSubRectangleString);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        // Disable any writes to the alpha channel
        gl.colorMask(1, 1, 1, 0);
        var texture = gl.createTexture();
        // Bind the texture to texture unit 0
        gl.bindTexture(bindingTarget, texture);
        // Set up texture parameters
        gl.texParameteri(bindingTarget, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        gl.texParameteri(bindingTarget, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        gl.texParameteri(bindingTarget, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(bindingTarget, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        // Set up pixel store parameters
        gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, flipY);
        gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, false);
        var targets = [gl.TEXTURE_2D];
        if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
            targets = [gl.TEXTURE_CUBE_MAP_POSITIVE_X,
                       gl.TEXTURE_CUBE_MAP_NEGATIVE_X,
                       gl.TEXTURE_CUBE_MAP_POSITIVE_Y,
                       gl.TEXTURE_CUBE_MAP_NEGATIVE_Y,
                       gl.TEXTURE_CUBE_MAP_POSITIVE_Z,
                       gl.TEXTURE_CUBE_MAP_NEGATIVE_Z];
        }
        // Handle the source sub-rectangle if specified (WebGL 2.0 only)
        if (sourceSubRectangle) {
            gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, sourceSubRectangle[0]);
            gl.pixelStorei(gl.UNPACK_SKIP_ROWS, sourceSubRectangle[1]);
        }
        // Upload the videoElement into the texture
        for (var tt = 0; tt < targets.length; ++tt) {
            if (sourceSubRectangle) {
                // Initialize the texture to black first
                if (useTexSubImage2D) {
                    // Skip sub-rectangle tests for cube map textures for the moment.
                    if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
                        continue;
                    }
                    gl.texImage2D(targets[tt], 0, gl[internalFormat],
                                  sourceSubRectangle[2], sourceSubRectangle[3], 0,
                                  gl[pixelFormat], gl[pixelType], null);
                    gl.texSubImage2D(targets[tt], 0, 0, 0,
                                     sourceSubRectangle[2], sourceSubRectangle[3],
                                     gl[pixelFormat], gl[pixelType], videoElement);
                } else {
                    gl.texImage2D(targets[tt], 0, gl[internalFormat],
                                  sourceSubRectangle[2], sourceSubRectangle[3], 0,
                                  gl[pixelFormat], gl[pixelType], videoElement);
                }
            } else {
                // Initialize the texture to black first
                if (useTexSubImage2D) {
                    var width = videoElement.videoWidth;
                    var height = videoElement.videoHeight;
                    if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
                        // cube map texture must be square.
                        width = Math.max(width, height);
                        height = width;
                    }
                    gl.texImage2D(targets[tt], 0, gl[internalFormat],
                                  width, height, 0,
                                  gl[pixelFormat], gl[pixelType], null);
                    gl.texSubImage2D(targets[tt], 0, 0, 0, gl[pixelFormat], gl[pixelType], videoElement);
                } else {
                    gl.texImage2D(targets[tt], 0, gl[internalFormat], gl[pixelFormat], gl[pixelType], videoElement);
                }
            }
        }

        if (sourceSubRectangle) {
            gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, 0);
            gl.pixelStorei(gl.UNPACK_SKIP_ROWS, 0);
        }

        var c = document.createElement("canvas");
        c.width = 16;
        c.height = 16;
        c.style.border = "1px solid black";
        var ctx = c.getContext("2d");
        ctx.drawImage(videoElement, 0, 0, 16, 16);
        document.body.appendChild(c);

        var loc;
        if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
            loc = gl.getUniformLocation(program, "face");
        }

        for (var tt = 0; tt < targets.length; ++tt) {
            if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
                gl.uniform1i(loc, targets[tt]);
            }
            // Draw the triangles
            wtu.clearAndDrawUnitQuad(gl, [0, 0, 0, 255]);
            // Check a few pixels near the top and bottom and make sure they have
            // the right color.
            var tolerance = currentTolerance;
            debug("Checking lower left corner");
            wtu.checkCanvasRect(gl, 4, 4, 2, 2, bottomColor,
                                "shouldBe " + bottomColor, tolerance);
            debug("Checking upper left corner");
            wtu.checkCanvasRect(gl, 4, gl.canvas.height - 8, 2, 2, topColor,
                                "shouldBe " + topColor, tolerance);

            // Expose bug http://crbug.com/733172.
            if (sourceSubRectangle) {
                // Skip sub-rectangle tests for cube map textures for the moment.
                if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
                    continue;
                }
                gl.texSubImage2D(targets[tt], 0, 0, 0,
                                 sourceSubRectangle[2], sourceSubRectangle[3],
                                 gl[pixelFormat], gl[pixelType], videoElement);
            } else {
                gl.texSubImage2D(targets[tt], 0, 0, 0, gl[pixelFormat], gl[pixelType], videoElement);
            }
            wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
        }
    }

    function runCanvas2DTest(videoElement, topColor, bottomColor)
    {
        debug('Testing with 2D canvas');

        var canvas = c2d.canvas;

        // Draw the video to the 2D canvas context.
        c2d.drawImage(videoElement, 0, 0, canvas.width, canvas.height);

        // Check a few pixels near the top and bottom and make sure they have
        // the right color.
        // Origin is upper left in 2D canvas context.
        var tolerance = currentTolerance;
        debug("Checking lower left corner");
        wtu.checkCanvasRect(c2d, 4, canvas.height - 8, 2, 2, bottomColor,
                            "shouldBe " + bottomColor, tolerance);
        debug("Checking upper left corner");
        wtu.checkCanvasRect(c2d, 4, 4, 2, 2, topColor,
                            "shouldBe " + topColor, tolerance);
    }

    function runAllTests()
    {
        var cases = [
            { sub: false, flipY: true, topColor: redColor, bottomColor: greenColor },
            { sub: false, flipY: false, topColor: greenColor, bottomColor: redColor },
            { sub: true, flipY: true, topColor: redColor, bottomColor: greenColor },
            { sub: true, flipY: false, topColor: greenColor, bottomColor: redColor },
        ];

        function runTexImageTest(bindingTarget) {
            var program;
            if (bindingTarget == gl.TEXTURE_2D) {
                program = tiu.setupTexturedQuad(gl, internalFormat);
            } else {
                program = tiu.setupTexturedQuadWithCubeMap(gl, internalFormat);
            }

            return new Promise(function(resolve, reject) {
                var videoNdx = 0;
                var video;
                function runNextVideo() {
                    if (video) {
                        video.pause();
                    }

                    if (videoNdx == videos.length) {
                        resolve("SUCCESS");
                        return;
                    }

                    var info = videos[videoNdx++];
                    debug("");
                    debug("testing: " + info.comment);
                    debug("video type: " + info.type);
                    // Default to tolerance of 5.
                    currentTolerance = info.tolerance || 5;
                    debug("tolerance: " + currentTolerance);
                    video = document.createElement("video");
                    var canPlay = true;
                    if (!video.canPlayType) {
                      testFailed("video.canPlayType required method missing");
                      runNextVideo();
                      return;
                    }

                    if(!video.canPlayType(info.type).replace(/no/, '')) {
                      debug(info.type + " unsupported; skipping test");
                      runNextVideo();
                      return;
                    };

                    document.body.appendChild(video);
                    video.type = info.type;
                    video.src = info.src;
                    wtu.startPlayingAndWaitForVideo(video, runTest);
                }
                function runTest() {
                    for (var i in cases) {
                        if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
                            // Cube map texture must be square but video is not square.
                            if (!cases[i].sub) {
                                break;
                            }
                            // Skip sub-rectangle tests for cube map textures for the moment.
                            if (cases[i].sourceSubRectangle) {
                                break;
                            }
                        }
                        runOneIteration(video, cases[i].sub, cases[i].flipY,
                                        cases[i].topColor, cases[i].bottomColor,
                                        cases[i].sourceSubRectangle,
                                        program, bindingTarget);
                    }
                    runCanvas2DTest(video, redColor, greenColor);
                    runNextVideo();
                }
                runNextVideo();
            });
        }

        runTexImageTest(gl.TEXTURE_2D).then(function(val) {
            wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
            finishTest();
        });
    }

    return init;
}

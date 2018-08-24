/*
** Copyright (c) 2015 The Khronos Group Inc.
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

function generateTest(internalFormat, pixelFormat, pixelType, prologue, resourcePath, defaultContextVersion) {
    var wtu = WebGLTestUtils;
    var tiu = TexImageUtils;
    var gl = null;
    var successfullyParsed = false;
    var redColor = [255, 0, 0];
    var greenColor = [0, 255, 0];

    // Test each format separately because many browsers implement each
    // differently. Some might be GPU accelerated, some might not. Etc...
    var videos = [
      { src: resourcePath + "red-green.mp4"         , type: 'video/mp4; codecs="avc1.42E01E, mp4a.40.2"', },
      { src: resourcePath + "red-green.bt601.vp9.webm", type: 'video/webm; codecs="vp9"',                   },
      { src: resourcePath + "red-green.webmvp8.webm", type: 'video/webm; codecs="vp8, vorbis"',           },
      { src: resourcePath + "red-green.theora.ogv",   type: 'video/ogg; codecs="theora, vorbis"',         },
    ];

    function init()
    {
        description('Verify texImage3D and texSubImage3D code paths taking video elements (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

        // Set the default context version while still allowing the webglVersion URL query string to override it.
        wtu.setDefault3DContextVersion(defaultContextVersion);
        gl = wtu.create3DContext("example");

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

        runTest();
    }

    function runOneIteration(videoElement, flipY, useTexSubImage3D, topColor, bottomColor, program, bindingTarget,
                             depth, sourceSubRectangle, unpackImageHeight, rTextureCoord)
    {
        debug('Testing ' +
              (useTexSubImage3D ? "texSubImage3D" : "texImage3D") +
              ' with flipY=' + flipY + ' bindingTarget=' +
              (bindingTarget == gl.TEXTURE_3D ? 'TEXTURE_3D' : 'TEXTURE_2D_ARRAY') +
              (sourceSubRectangle ? ', sourceSubRectangle=' + sourceSubRectangle : '') +
              (unpackImageHeight ? ', unpackImageHeight=' + unpackImageHeight : '') +
              ', depth=' + depth +
              ', rTextureCoord=' + rTextureCoord);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        // Disable any writes to the alpha channel
        gl.colorMask(1, 1, 1, 0);
        var texture = gl.createTexture();
        // Bind the texture to texture unit 0
        gl.bindTexture(bindingTarget, texture);
        // Set up texture parameters
        gl.texParameteri(bindingTarget, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        gl.texParameteri(bindingTarget, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        gl.texParameteri(bindingTarget, gl.TEXTURE_WRAP_R, gl.CLAMP_TO_EDGE);
        gl.texParameteri(bindingTarget, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(bindingTarget, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        // Set up pixel store parameters
        gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, flipY);
        gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, false);
        gl.pixelStorei(gl.UNPACK_COLORSPACE_CONVERSION_WEBGL, gl.NONE);
        var uploadWidth = videoElement.width;
        var uploadHeight = videoElement.height;
        if (sourceSubRectangle) {
            gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, sourceSubRectangle[0]);
            gl.pixelStorei(gl.UNPACK_SKIP_ROWS, sourceSubRectangle[1]);
            uploadWidth = sourceSubRectangle[2];
            uploadHeight = sourceSubRectangle[3];
        }
        if (unpackImageHeight) {
            gl.pixelStorei(gl.UNPACK_IMAGE_HEIGHT, unpackImageHeight);
        }
        // Upload the videoElement into the texture
        if (useTexSubImage3D) {
            // Initialize the texture to black first
            gl.texImage3D(bindingTarget, 0, gl[internalFormat],
                          uploadWidth, uploadHeight, depth, 0,
                          gl[pixelFormat], gl[pixelType], null);
            gl.texSubImage3D(bindingTarget, 0, 0, 0, 0,
                             uploadWidth, uploadHeight, depth,
                             gl[pixelFormat], gl[pixelType], videoElement);
        } else {
            gl.texImage3D(bindingTarget, 0, gl[internalFormat],
                          uploadWidth, uploadHeight, depth, 0,
                          gl[pixelFormat], gl[pixelType], videoElement);
        }
        gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, 0);
        gl.pixelStorei(gl.UNPACK_SKIP_ROWS, 0);
        gl.pixelStorei(gl.UNPACK_IMAGE_HEIGHT, 0);

        var c = document.createElement("canvas");
        c.width = 16;
        c.height = 16;
        c.style.border = "1px solid black";
        var ctx = c.getContext("2d");
        ctx.drawImage(videoElement, 0, 0, 16, 16);
        document.body.appendChild(c);

        var rCoordLocation = gl.getUniformLocation(program, 'uRCoord');
        if (!rCoordLocation) {
            testFailed('Shader incorrectly set up; couldn\'t find uRCoord uniform');
            return;
        }
        gl.uniform1f(rCoordLocation, rTextureCoord);

        // Draw the triangles
        wtu.clearAndDrawUnitQuad(gl, [0, 0, 0, 255]);
        // Check a few pixels near the top and bottom and make sure they have
        // the right color.
        var tolerance = 5;
        debug("Checking lower left corner");
        wtu.checkCanvasRect(gl, 4, 4, 2, 2, bottomColor,
                            "shouldBe " + bottomColor, tolerance);
        debug("Checking upper left corner");
        wtu.checkCanvasRect(gl, 4, gl.canvas.height - 8, 2, 2, topColor,
                            "shouldBe " + topColor, tolerance);
    }

    function runTest(videoElement)
    {
        var cases = [
            // No UNPACK_IMAGE_HEIGHT specified.
            { flipY: false, sourceSubRectangle: [32, 16, 16, 16], depth: 5, rTextureCoord: 0,
              topColor: redColor, bottomColor: redColor },
            // Note that an rTextureCoord of 4.0 satisfies the need to
            // have it be >= 1.0 for the TEXTURE_3D case, and also its
            // use as an index in the TEXTURE_2D_ARRAY case.
            { flipY: false, sourceSubRectangle: [32, 16, 16, 16], depth: 5, rTextureCoord: 4,
              topColor: greenColor, bottomColor: greenColor },
            { flipY: false, sourceSubRectangle: [24, 48, 32, 32], depth: 1, rTextureCoord: 0,
              topColor: greenColor, bottomColor: redColor },
            { flipY: true, sourceSubRectangle: [24, 48, 32, 32], depth: 1, rTextureCoord: 0,
              topColor: redColor, bottomColor: greenColor },

            // Use UNPACK_IMAGE_HEIGHT to skip some pixels.
            { flipY: false, sourceSubRectangle: [32, 16, 16, 16], depth: 2, unpackImageHeight: 64, rTextureCoord: 0,
              topColor: redColor, bottomColor: redColor },
            { flipY: false, sourceSubRectangle: [32, 16, 16, 16], depth: 2, unpackImageHeight: 64, rTextureCoord: 1,
              topColor: greenColor, bottomColor: greenColor },
        ];

        function runTexImageTest(bindingTarget) {
            var program;
            if (bindingTarget == gl.TEXTURE_3D) {
                program = tiu.setupTexturedQuadWith3D(gl, internalFormat);
            } else {
                program = tiu.setupTexturedQuadWith2DArray(gl, internalFormat);
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
                    debug("testing: " + info.type);
                    video = document.createElement("video");
                    var canPlay = true;
                    if (!video.canPlayType) {
                      testFailed("video.canPlayType required method missing");
                      runNextVideo();
                      return;
                    }

                    if(!video.canPlayType(info.type).replace(/no/, '')) {
                      debug(info.type + " unsupported");
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
                        runOneIteration(video, cases[i].flipY, false,
                                        cases[i].topColor, cases[i].bottomColor,
                                        program, bindingTarget, cases[i].depth,
                                        cases[i].sourceSubRectangle,
                                        cases[i].unpackImageHeight,
                                        cases[i].rTextureCoord);
                        runOneIteration(video, cases[i].flipY, true,
                                        cases[i].topColor, cases[i].bottomColor,
                                        program, bindingTarget, cases[i].depth,
                                        cases[i].sourceSubRectangle,
                                        cases[i].unpackImageHeight,
                                        cases[i].rTextureCoord);
                    }
                    runNextVideo();
                }
                runNextVideo();
            });
        }

        runTexImageTest(gl.TEXTURE_3D).then(function(val) {
            runTexImageTest(gl.TEXTURE_2D_ARRAY).then(function(val) {
                wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
                finishTest();
            });
        });
    }

    return init;
}

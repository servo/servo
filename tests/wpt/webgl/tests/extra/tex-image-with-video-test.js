/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

// This block needs to be outside the onload handler in order for this
// test to run reliably in WebKit's test harness (at least the
// Chromium port). https://bugs.webkit.org/show_bug.cgi?id=87448
initTestingHarness();

var old = debug;
var debug = function(msg) {
  console.log(msg);
  old(msg);
};

function generateTest(pixelFormat, pixelType, prologue) {
    var wtu = WebGLTestUtils;
    var gl = null;
    var textureLoc = null;
    var successfullyParsed = false;

    // Test each format separately because many browsers implement each
    // differently. Some might be GPU accelerated, some might not. Etc...
    var videos = [
      { src: "../resources/red-green.mp4"         , type: 'video/mp4; codecs="avc1.42E01E, mp4a.40.2"', },
      { src: "../resources/red-green.webmvp8.webm", type: 'video/webm; codecs="vp8, vorbis"',           },
      { src: "../resources/red-green.webmvp9.webm", type: 'video/webm; codecs="vp9"',                   },
    ];

    var videoNdx = 0;
    var video;

    function runNextVideo() {
        if (video) {
            video.pause();
        }

        if (videoNdx == videos.length) {
            finishTest();
            return;
        }

        var info = videos[videoNdx++];
        debug("");
        debug("testing: " + info.type);
        video = document.createElement("video");
        video.muted = true;
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
        video.crossOrigin = 'anonymous';
        video.src = info.src;
        wtu.startPlayingAndWaitForVideo(video, runTest);
    }

    var init = function()
    {
        description('Verify texImage2D and texSubImage2D code paths taking video elements (' + pixelFormat + '/' + pixelType + ')');

        gl = wtu.create3DContext("example");

        if (!prologue(gl)) {
            finishTest();
            return;
        }

        var program = wtu.setupTexturedQuad(gl);

        gl.clearColor(0,0,0,1);
        gl.clearDepth(1);

        textureLoc = gl.getUniformLocation(program, "tex");
        runNextVideo();
    }

    function runOneIteration(videoElement, useTexSubImage2D, flipY, topColor, bottomColor)
    {
        debug('Testing ' + (useTexSubImage2D ? 'texSubImage2D' : 'texImage2D') +
              ' with flipY=' + flipY);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        // Disable any writes to the alpha channel
        gl.colorMask(1, 1, 1, 0);
        var texture = gl.createTexture();
        // Bind the texture to texture unit 0
        gl.bindTexture(gl.TEXTURE_2D, texture);
        // Set up texture parameters
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        // Set up pixel store parameters
        gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, flipY);
        gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, false);
        // Upload the videoElement into the texture
        if (useTexSubImage2D) {
            // Initialize the texture to black first
            gl.texImage2D(gl.TEXTURE_2D, 0, gl[pixelFormat],
                          videoElement.videoWidth, videoElement.videoHeight, 0,
                          gl[pixelFormat], gl[pixelType], null);
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, gl[pixelFormat], gl[pixelType], videoElement);
        } else {
            gl.texImage2D(gl.TEXTURE_2D, 0, gl[pixelFormat], gl[pixelFormat], gl[pixelType], videoElement);
        }

        var c = document.createElement("canvas");
        c.width = 16;
        c.height = 16;
        c.style.border = "1px solid black";
        var ctx = c.getContext("2d");
        ctx.drawImage(videoElement, 0, 0, 16, 16);
        document.body.appendChild(c);

        // Point the uniform sampler to texture unit 0
        gl.uniform1i(textureLoc, 0);
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
        var red = [255, 0, 0];
        var green = [0, 255, 0];
        runOneIteration(videoElement, false, true, red, green);
        runOneIteration(videoElement, false, false, green, red);
        runOneIteration(videoElement, true, true, red, green);
        runOneIteration(videoElement, true, false, green, red);

        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");

        runNextVideo();
    }

    return init;
}

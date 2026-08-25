/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

function generateTest(internalFormat, pixelFormat, pixelType, prologue, resourcePath, defaultContextVersion) {
    var wtu = WebGLTestUtils;
    var tiu = TexImageUtils;
    var gl = null;
    var successfullyParsed = false;

    var videos = [
        { src: resourcePath + "red-green.mp4"           , type: 'video/mp4; codecs="avc1.42E01E, mp4a.40.2"', },
        { src: resourcePath + "red-green.webmvp8.webm"  , type: 'video/webm; codecs="vp8, vorbis"',           },
        { src: resourcePath + "red-green.bt601.vp9.webm", type: 'video/webm; codecs="vp9"',                   },
    ];

    function init()
    {
        description('Verify texImage3D and texSubImage3D code paths taking ImageBitmap created from an HTMLVideoElement (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

        if(!window.createImageBitmap || !window.ImageBitmap) {
            finishTest();
            return;
        }

        // Set the default context version while still allowing the webglVersion URL query string to override it.
        wtu.setDefault3DContextVersion(defaultContextVersion);
        gl = wtu.create3DContext("example");

        if (!prologue(gl)) {
            finishTest();
            return;
        }

        gl.clearColor(0,0,0,1);
        gl.clearDepth(1);

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
            video.src = info.src;
            wtu.startPlayingAndWaitForVideo(video, async function() {
                await runImageBitmapTest(video, 1, internalFormat, pixelFormat, pixelType, gl, tiu, wtu, true);
                runNextVideo();
            });
        }
        runNextVideo();
    }

    return init;
}

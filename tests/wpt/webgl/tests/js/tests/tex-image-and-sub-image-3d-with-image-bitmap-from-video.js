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

function generateTest(internalFormat, pixelFormat, pixelType, prologue, resourcePath, defaultContextVersion) {
    var wtu = WebGLTestUtils;
    var tiu = TexImageUtils;
    var gl = null;
    var successfullyParsed = false;

    var videos = [
        { src: resourcePath + "red-green.mp4"           , type: 'video/mp4; codecs="avc1.42E01E, mp4a.40.2"', },
        { src: resourcePath + "red-green.webmvp8.webm"  , type: 'video/webm; codecs="vp8, vorbis"',           },
        { src: resourcePath + "red-green.bt601.vp9.webm", type: 'video/webm; codecs="vp9"',                   },
        { src: resourcePath + "red-green.theora.ogv"    , type: 'video/ogg; codecs="theora, vorbis"',         },
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
            wtu.startPlayingAndWaitForVideo(video, function() {
                runImageBitmapTest(video, 1, internalFormat, pixelFormat, pixelType, gl, tiu, wtu, true);
                runNextVideo();
            });
        }
        runNextVideo();
    }

    return init;
}

function makeCanvas() {
    return new Promise(resolve => {
        var testCanvas = document.createElement("canvas");
        testCanvas.width = 20;
        testCanvas.height = 20;
        var testCtx = testCanvas.getContext("2d");
        testCtx.fillStyle = "rgb(255, 0, 0)";
        testCtx.fillRect(0, 0, 10, 10);
        testCtx.fillStyle = "rgb(0, 255, 0)";
        testCtx.fillRect(10, 0, 10, 10);
        testCtx.fillStyle = "rgb(0, 0, 255)";
        testCtx.fillRect(0, 10, 10, 10);
        testCtx.fillStyle = "rgb(0, 0, 0)";
        testCtx.fillRect(10, 10, 10, 10);
        resolve(testCanvas);
    });
}

function makeOffscreenCanvas() {
    return new Promise(resolve => {
        let canvas = new OffscreenCanvas(20, 20);
        var testCtx = canvas.getContext("2d");
        testCtx.fillStyle = "rgb(255, 0, 0)";
        testCtx.fillRect(0, 0, 10, 10);
        testCtx.fillStyle = "rgb(0, 255, 0)";
        testCtx.fillRect(10, 0, 10, 10);
        testCtx.fillStyle = "rgb(0, 0, 255)";
        testCtx.fillRect(0, 10, 10, 10);
        testCtx.fillStyle = "rgb(0, 0, 0)";
        testCtx.fillRect(10, 10, 10, 10);
        resolve(canvas);
    });
}

var imageBitmapVideoPromise = new Promise(function(resolve, reject) {
    var video = document.createElement("video");
    video.oncanplaythrough = function() {
        resolve(video);
    };
    video.onerror = reject;
    video.src = getVideoURI("/images/pattern");

    // Prevent WebKit from garbage collecting event handlers.
    window._video = video;
});

function makeVideo() {
    return imageBitmapVideoPromise;
}

var imageBitmapDataUrlVideoPromise = fetch(getVideoURI("/images/pattern"))
    .then(response => Promise.all([response.headers.get("Content-Type"), response.arrayBuffer()]))
    .then(([type, data]) => {
        return new Promise(function(resolve, reject) {
            var video = document.createElement("video");
            video.oncanplaythrough = function() {
                resolve(video);
            };
            video.onerror = reject;

            var encoded = btoa(String.fromCodePoint(...new Uint8Array(data)));
            var dataUrl = `data:${type};base64,${encoded}`;
            video.src = dataUrl;

            // Prevent WebKit from garbage collecting event handlers.
            window._dataVideo = video;
        });
    });

function makeDataUrlVideo() {
    return imageBitmapDataUrlVideoPromise;
}

function makeMakeHTMLImage(src) {
    return function() {
        return new Promise((resolve, reject) => {
            var img = new Image();
            img.onload = function() {
                resolve(img);
            };
            img.onerror = reject;
            img.src = src;
        });
    }
}

function makeMakeSVGImage(src) {
    return function() {
        return new Promise((resolve, reject) => {
            var image = document.createElementNS(NAMESPACES.svg, "image");
            image.onload = () => resolve(image);
            image.onerror = reject;
            image.setAttribute("externalResourcesRequired", "true");
            image.setAttributeNS(NAMESPACES.xlink, 'xlink:href', src);
            document.body.appendChild(image);
        });
    }
}

function makeImageData() {
    return new Promise(function(resolve, reject) {
        var width = 20, height = 20;
        var imgData = new ImageData(width, height);
        for (var i = 0; i < width * height * 4; i += 4) {
            imgData.data[i] = 0;
            imgData.data[i + 1] = 0;
            imgData.data[i + 2] = 0;
            imgData.data[i + 3] = 255; //alpha channel: 255
        }
        var halfWidth = width / 2;
        var halfHeight = height / 2;
        // initialize to R, G, B, Black, with each one 10*10 pixels
        for (var i = 0; i < halfHeight; i++)
            for (var j = 0; j < halfWidth; j++)
                imgData.data[i * width * 4 + j * 4] = 255;
        for (var i = 0; i < halfHeight; i++)
            for (var j = halfWidth; j < width; j++)
                imgData.data[i * width * 4 + j * 4 + 1] = 255;
        for (var i = halfHeight; i < height; i++)
            for (var j = 0; j < halfWidth; j++)
                imgData.data[i * width * 4 + j * 4 + 2] = 255;
        resolve(imgData);
    });
}

function makeImageBitmap() {
    return makeCanvas().then(canvas => {
        return createImageBitmap(canvas);
    });
}

function makeBlob() {
    return new Promise(function(resolve, reject) {
        var xhr = new XMLHttpRequest();
        xhr.open("GET", '/images/pattern.png');
        xhr.responseType = 'blob';
        xhr.send();
        xhr.onload = function() {
            resolve(xhr.response);
        };
    });
}

var imageSourceTypes = [
    { name: 'an HTMLCanvasElement', factory: makeCanvas },
    { name: 'an HTMLVideoElement',  factory: makeVideo },
    { name: 'an HTMLVideoElement from a data URL', factory: makeDataUrlVideo },
    { name: 'a bitmap HTMLImageElement', factory: makeMakeHTMLImage("/images/pattern.png") },
    { name: 'a vector HTMLImageElement', factory: makeMakeHTMLImage("/images/pattern.svg") },
    { name: 'a bitmap SVGImageElement', factory: makeMakeSVGImage("/images/pattern.png") },
    { name: 'a vector SVGImageElement', factory: makeMakeSVGImage("/images/pattern.svg") },
    { name: 'an OffscreenCanvas',   factory: makeOffscreenCanvas },
    { name: 'an ImageData',         factory: makeImageData },
    { name: 'an ImageBitmap',       factory: makeImageBitmap },
    { name: 'a Blob',               factory: makeBlob },
];

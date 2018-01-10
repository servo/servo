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

function makeVideo() {
    return new Promise(function(resolve, reject) {
        var video = document.createElement("video");
        video.oncanplaythrough = function() {
            resolve(video);
        };
        video.src = "/images/pattern.ogv";
    });
}

function makeImage() {
    return new Promise(resolve => {
        var img = new Image();
        img.onload = function() {
            resolve(img);
        };
        img.src = "/images/pattern.png";
    });
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
    { name: 'an HTMLImageElement',  factory: makeImage },
    { name: 'an OffscreenCanvas',   factory: makeOffscreenCanvas },
    { name: 'an ImageData',         factory: makeImageData },
    { name: 'an ImageBitmap',       factory: makeImageBitmap },
    { name: 'a Blob',               factory: makeBlob },
];

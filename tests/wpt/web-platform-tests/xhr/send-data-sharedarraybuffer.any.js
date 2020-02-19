// META: title=XMLHttpRequest.send(sharedarraybuffer)

test(() => {
    var xhr = new XMLHttpRequest();
    var buf = new SharedArrayBuffer();

    xhr.open("POST", "./resources/content.py", true);
    assert_throws_js(TypeError, function() {
        xhr.send(buf)
    });
}, "sending a SharedArrayBuffer");

["Int8Array", "Uint8Array", "Uint8ClampedArray", "Int16Array", "Uint16Array",
 "Int32Array", "Uint32Array", "Float32Array", "Float64Array", "DataView"].forEach((type) => {
    test(() => {
        var xhr = new XMLHttpRequest();
        var arr = new self[type](new SharedArrayBuffer());

        xhr.open("POST", "./resources/content.py", true);
        assert_throws_js(TypeError, function() {
            xhr.send(arr)
        });
    }, `sending a ${type} backed by a SharedArrayBuffer`);
});

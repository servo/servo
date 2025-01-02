// META: title=XMLHttpRequest.send(sharedarraybuffer)

test(() => {
    const xhr = new XMLHttpRequest();
    // See https://github.com/whatwg/html/issues/5380 for why not `new SharedArrayBuffer()`
    const buf = new WebAssembly.Memory({ shared:true, initial:1, maximum:1 }).buffer;

    xhr.open("POST", "./resources/content.py", true);
    assert_throws_js(TypeError, function() {
        xhr.send(buf)
    });
}, "sending a SharedArrayBuffer");

["Int8Array", "Uint8Array", "Uint8ClampedArray", "Int16Array", "Uint16Array",
 "Int32Array", "Uint32Array", "BigInt64Array", "BigUint64Array",
 "Float16Array", "Float32Array", "Float64Array", "DataView"].forEach((type) => {
    test(() => {
        const xhr = new XMLHttpRequest();
        // See https://github.com/whatwg/html/issues/5380 for why not `new SharedArrayBuffer()`
        const arr = new self[type](new WebAssembly.Memory({ shared:true, initial:1, maximum:1 }).buffer);

        xhr.open("POST", "./resources/content.py", true);
        assert_throws_js(TypeError, function() {
            xhr.send(arr)
        });
    }, `sending a ${type} backed by a SharedArrayBuffer`);
});

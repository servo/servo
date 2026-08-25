// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
    // `empty.js` is the 15-byte "// Empty script" resource. These are its
    // stable subresource integrity digests.
    const SHA256 = "sha256-Ob5toHqY7M191fMXYeLJWkPa3s6KKUJG56w6yx2Va9g=";
    const SHA384 =
        "sha384-l9bPabkzR+5OF8EAN7erMPo/2NdApkGLXblgglaCaoG0NipjZ/+MitE1db4KmHTk";
    const preloads = [
        {
            "url": "empty.js?" + token(),
            "as_attr": "script",
            "integrity_attr": SHA256,
        },
        {
            "url": "empty.js?" + token(),
            "as_attr": "script",
            "integrity_attr": SHA384,
        },
    ];
    navigateToTestWithEarlyHints("resources/preload-integrity.html", preloads);
});

// META: title=XMLHttpRequest.send(arraybufferview)

var test = async_test();
test.step(function()
{
    var str = "Hello";
    var bytes = str.split("").map(function(ch) { return ch.charCodeAt(0); });
    var xhr = new XMLHttpRequest();
    var arr = new Uint8Array(bytes);

    xhr.onload = test.step_func_done(function() {
        assert_equals(xhr.status, 200);
        assert_equals(xhr.response, str);
    });

    xhr.open("POST", "./resources/content.py", true);
    xhr.send(arr);
});

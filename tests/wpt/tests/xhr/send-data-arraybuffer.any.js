// META: title=XMLHttpRequest.send(arraybuffer)

var test = async_test();
test.step(function()
{
    var xhr = new XMLHttpRequest();
    var buf = new ArrayBuffer(5);
    var arr = new Uint8Array(buf);
    arr[0] = 0x48;
    arr[1] = 0x65;
    arr[2] = 0x6c;
    arr[3] = 0x6c;
    arr[4] = 0x6f;

    xhr.onreadystatechange = function()
    {
        if (xhr.readyState == 4)
        {
            test.step(function()
            {
                assert_equals(xhr.status, 200);
                assert_equals(xhr.response, "Hello");

                test.done();
            });
        }
    };

    xhr.open("POST", "./resources/content.py", true);
    xhr.send(buf);
});

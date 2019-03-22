// META: title=XMLHttpRequest.send(formdata)

var test = async_test();
test.step(function()
{
    var xhr = new XMLHttpRequest();
    var form = new FormData();
    form.append("id", "0");
    form.append("value", "zero");

    xhr.onreadystatechange = test.step_func(() => {
        if (xhr.readyState == 4) {
            assert_equals(xhr.status, 200);
            assert_equals(xhr.response, "id:0;value:zero;");
            test.done();
        }
    });

    xhr.open("POST", "./resources/form.py", true);
    xhr.send(form);
});

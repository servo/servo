// META: title=XMLHttpRequest: The abort() method: Fire a progress event named loadend

        var test = async_test(function(test)
        {
            var xhr = new XMLHttpRequest();

            xhr.onloadstart = function()
            {
                test.step(function()
                {
                    if (xhr.readyState == 1)
                    {
                        xhr.abort();
                    }
                });
            };

            xhr.onloadend = function(e)
            {
                test.step(function()
                {
                    assert_true(e instanceof ProgressEvent);
                    assert_equals(e.type, "loadend");
                    test.done();
                });
            };

            xhr.open("GET", "resources/content.py", true);
            xhr.send();
        });

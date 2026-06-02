// META: title=XMLHttpRequest: The abort() method: do not fire abort event in OPENED state when send() flag is unset.

        var test = async_test()

        test.step(function()
        {
            var xhr = new XMLHttpRequest()

            xhr.onreadystatechange = function()
            {
                test.step(function()
                {
                    if (xhr.readyState == 1)
                    {
                        xhr.abort();
                        assert_equals(xhr.readyState, 1, "abort() cannot change readyState when readyState is 1 and send() flag is unset")
                    }
                });
            };

            xhr.onabort = function(e)
            {
                test.step(function()
                {
                    assert_unreached('when abort() is called, state is OPENED with the send() flag being unset, must not fire abort event per spec')
                });
            };

            xhr.open("GET", "./resources/content.py", true); // This should cause a readystatechange event that calls abort()
            xhr.send() // should not throw since abort() was a no-op
            test.done()
        });

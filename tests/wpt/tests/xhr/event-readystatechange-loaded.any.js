// META: title=XMLHttpRequest: the LOADING state change may be emitted multiple times

var test = async_test();

test.step(function () {
    var client = new XMLHttpRequest();
    var countedLoading = 0;

    client.onreadystatechange = test.step_func(function () {
        if (client.readyState === 3) {
            countedLoading += 1;
        }

        if (client.readyState === 4) {
            assert_greater_than(countedLoading, 1, "LOADING state change may be emitted multiple times");

            test.done();
        }
    });

    client.open("GET", "resources/trickle.py?count=10"); // default timeout in trickle.py is 1/2 sec, so this request will take 5 seconds to complete
    client.send(null);
});

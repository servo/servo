interval = setInterval('{', 10);
step_timeout(function(){
    clearInterval(interval);
    t.step(function(){
        assert_true(ran, 'ran');
        t.done();
    });
}, 20);
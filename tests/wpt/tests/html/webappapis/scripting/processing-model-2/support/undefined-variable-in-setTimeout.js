setTimeout('undefined_variable;', 10);
setTimeout(function(){
    t.step(function(){
        assert_true(ran, 'ran');
        t.done();
    });
}, 20);

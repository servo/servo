from tests.support.asserts import assert_error, assert_success


def execute_script(session, script, args=None):
    if args is None:
        args = []
    body = {"script": script, "args": args}

    return session.transport.send(
        "POST", "/session/{session_id}/execute/sync".format(
            session_id=session.session_id),
        body)


def test_promise_resolve(session):
    response = execute_script(session, """
        return Promise.resolve('foobar');
        """)
    assert_success(response, "foobar")


def test_promise_resolve_delayed(session):
    response = execute_script(session, """
        return new Promise(
            (resolve) => setTimeout(
                () => resolve('foobar'),
                50
            )
        );
        """)
    assert_success(response, "foobar")


def test_promise_all_resolve(session):
    response = execute_script(session, """
        return Promise.all([
            Promise.resolve(1),
            Promise.resolve(2)
        ]);
        """)
    assert_success(response, [1, 2])


def test_await_promise_resolve(session):
    response = execute_script(session, """
        let res = await Promise.resolve('foobar');
        return res;
        """)
    assert_success(response, "foobar")


def test_promise_resolve_timeout(session):
    session.timeouts.script = .1
    response = execute_script(session, """
        return new Promise(
            (resolve) => setTimeout(
                () => resolve(),
                1000
            )
        );
        """)
    assert_error(response, "script timeout")


def test_promise_reject(session):
    response = execute_script(session, """
        return Promise.reject(new Error('my error'));
        """)
    assert_error(response, "javascript error")


def test_promise_reject_delayed(session):
    response = execute_script(session, """
        return new Promise(
            (resolve, reject) => setTimeout(
                () => reject(new Error('my error')),
                50
            )
        );
        """)
    assert_error(response, "javascript error")


def test_promise_all_reject(session):
    response = execute_script(session, """
        return Promise.all([
            Promise.resolve(1),
            Promise.reject(new Error('error'))
        ]);
        """)
    assert_error(response, "javascript error")


def test_await_promise_reject(session):
    response = execute_script(session, """
        await Promise.reject(new Error('my error'));
        return 'foo';
        """)
    assert_error(response, "javascript error")


def test_promise_reject_timeout(session):
    session.timeouts.script = .1
    response = execute_script(session, """
        return new Promise(
            (resolve, reject) => setTimeout(
                () => reject(new Error('my error')),
                1000
            )
        );
        """)
    assert_error(response, "script timeout")

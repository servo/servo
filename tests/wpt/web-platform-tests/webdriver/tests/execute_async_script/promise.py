from tests.support.asserts import assert_error, assert_success
from . import execute_async_script


def test_promise_resolve(session):
    response = execute_async_script(session, """
        let resolve = arguments[0];
        resolve(Promise.resolve('foobar'));
        """)
    assert_success(response, "foobar")


def test_promise_resolve_delayed(session):
    response = execute_async_script(session, """
        let resolve = arguments[0];
        let promise = new Promise(
            (resolve) => setTimeout(
                () => resolve('foobar'),
                50
            )
        );
        resolve(promise);
        """)
    assert_success(response, "foobar")


def test_promise_all_resolve(session):
    response = execute_async_script(session, """
        let resolve = arguments[0];
        let promise = Promise.all([
            Promise.resolve(1),
            Promise.resolve(2)
        ]);
        resolve(promise);
        """)
    assert_success(response, [1, 2])


def test_await_promise_resolve(session):
    response = execute_async_script(session, """
        let resolve = arguments[0];
        let res = await Promise.resolve('foobar');
        resolve(res);
        """)
    assert_success(response, "foobar")


def test_promise_resolve_timeout(session):
    session.timeouts.script = .1
    response = execute_async_script(session, """
        let resolve = arguments[0];
        let promise = new Promise(
            (resolve) => setTimeout(
                () => resolve(),
                1000
            )
        );
        resolve(promise);
        """)
    assert_error(response, "script timeout")


def test_promise_reject(session):
    response = execute_async_script(session, """
        let resolve = arguments[0];
        resolve(Promise.reject(new Error('my error')));
        """)
    assert_error(response, "javascript error")


def test_promise_reject_delayed(session):
    response = execute_async_script(session, """
        let resolve = arguments[0];
        let promise = new Promise(
            (resolve, reject) => setTimeout(
                () => reject(new Error('my error')),
                50
            )
        );
        resolve(promise);
        """)
    assert_error(response, "javascript error")


def test_promise_all_reject(session):
    response = execute_async_script(session, """
        let resolve = arguments[0];
        let promise = Promise.all([
            Promise.resolve(1),
            Promise.reject(new Error('error'))
        ]);
        resolve(promise);
        """)
    assert_error(response, "javascript error")


def test_await_promise_reject(session):
    response = execute_async_script(session, """
        let resolve = arguments[0];
        await Promise.reject(new Error('my error'));
        resolve('foo');
        """)
    assert_error(response, "javascript error")


def test_promise_reject_timeout(session):
    session.timeouts.script = .1
    response = execute_async_script(session, """
        let resolve = arguments[0];
        let promise = new Promise(
            (resolve, reject) => setTimeout(
                () => reject(new Error('my error')),
                1000
            )
        );
        resolve(promise);
        """)
    assert_error(response, "script timeout")

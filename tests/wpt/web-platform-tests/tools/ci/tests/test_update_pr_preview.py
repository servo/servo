try:
    from BaseHTTPServer import BaseHTTPRequestHandler, HTTPServer
except ImportError:
    # Python 3 case
    from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os
import subprocess
import sys
import tempfile
import threading

import pytest

subject = os.path.join(
    os.path.dirname(os.path.abspath(__file__)), '..', 'update_pr_preview.py'
)
test_host = 'localhost'


class MockHandler(BaseHTTPRequestHandler, object):
    def do_all(self):
        request_body = None

        if 'Content-Length' in self.headers:
            request_body = self.rfile.read(
                int(self.headers['Content-Length'])
            ).decode('utf-8')

            if self.headers.get('Content-Type') == 'application/json':
                request_body = json.loads(request_body)

        request = (self.command, self.path, request_body)

        self.server.requests.append(request)
        status_code, body = self.server.responses.get(request[:2], (200, '{}'))
        self.send_response(status_code)
        self.end_headers()
        self.wfile.write(body.encode('utf-8'))

    def do_DELETE(self):
        return self.do_all()

    def do_GET(self):
        return self.do_all()

    def do_PATCH(self):
        return self.do_all()

    def do_POST(self):
        return self.do_all()


class MockServer(HTTPServer, object):
    '''HTTP server that responds to all requests with status code 200 and body
    '{}' unless an alternative status code and body are specified for the given
    method and path in the `responses` parameter.'''
    def __init__(self, address, responses=None):
        super(MockServer, self).__init__(address, MockHandler)
        self.responses = responses or {}
        self.requests = []


def assert_success(returncode):
    assert returncode == 0


def assert_neutral(returncode):
    assert returncode == 78


def assert_fail(returncode):
    assert returncode not in (0, 78)


def run(event_data, responses=None):
    event_data_file = tempfile.mkstemp()[1]
    env = {
        'GITHUB_EVENT_PATH': event_data_file,
        'GITHUB_REPOSITORY': 'test-org/test-repo'
    }
    env.update(os.environ)
    server = MockServer((test_host, 0), responses)
    test_port = server.server_address[1]
    threading.Thread(target=lambda: server.serve_forever()).start()

    try:
        with open(event_data_file, 'w') as handle:
            json.dump(event_data, handle)

        child = subprocess.Popen(
            ['python', subject, 'http://{}:{}'.format(test_host, test_port)],
            env=env
        )

        child.communicate()
    finally:
        server.shutdown()
        os.remove(event_data_file)

    return child.returncode, server.requests

def to_key(request):
    return request[:2]

class Requests(object):
    read_collaborator = (
        'GET', '/repos/test-org/test-repo/collaborators/rms', None
    )
    create_label = (
        'POST',
        '/repos/test-org/test-repo/issues/543/labels',
        {'labels': ['pull-request-has-preview']}
    )
    delete_label = (
        'DELETE',
        '/repos/test-org/test-repo/issues/543/labels/pull-request-has-preview',
        ''
    )
    get_ref_open = (
        'GET', '/repos/test-org/test-repo/git/refs/prs-open/gh-543', None
    )
    get_ref_labeled = (
        'GET',
        '/repos/test-org/test-repo/git/refs/prs-labeled-for-preview/gh-543',
        None
    )
    create_ref_open = (
        'POST',
        '/repos/test-org/test-repo/git/refs',
        {'ref': 'refs/prs-open/gh-543', 'sha': 'deadbeef'}
    )
    create_ref_labeled = (
        'POST',
        '/repos/test-org/test-repo/git/refs',
        {'ref': 'refs/prs-labeled-for-preview/gh-543', 'sha': 'deadbeef'}
    )
    delete_ref_open = (
        'DELETE', '/repos/test-org/test-repo/git/refs/prs-open/gh-543', ''
    )
    delete_ref_labeled = (
        'DELETE',
        '/repos/test-org/test-repo/git/refs/prs-labeled-for-preview/gh-543', ''
    )
    update_ref_open = (
        'PATCH',
        '/repos/test-org/test-repo/git/refs/prs-open/gh-543',
        {'force': True, 'sha': 'deadbeef'}
    )
    update_ref_labeled = (
        'PATCH',
        '/repos/test-org/test-repo/git/refs/prs-labeled-for-preview/gh-543',
        {'force': True, 'sha': 'deadbeef'}
    )


def default_data(action):
    return {
        'pull_request': {
            'number': 543,
            'closed_at': None,
            'head': {
                'sha': 'deadbeef'
            },
            'user': {
                'login': 'rms'
            },
            'labels': [
                {'name': 'foo'},
                {'name': 'bar'}
            ]
        },
        'action': action
    }


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/18255")
def test_close_active_with_label():
    event_data = default_data('closed')
    event_data['pull_request']['closed_at'] = '2019-07-05'
    event_data['pull_request']['labels'].append(
        {'name': 'pull-request-has-preview'}
    )

    returncode, requests = run(event_data)

    assert_success(returncode)
    assert Requests.delete_label in requests
    assert Requests.delete_ref_open in requests
    assert Requests.delete_ref_labeled not in requests


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/18255")
def test_close_active_with_label_error():
    event_data = default_data('closed')
    event_data['pull_request']['closed_at'] = '2019-07-05'
    event_data['pull_request']['labels'].append(
        {'name': 'pull-request-has-preview'}
    )
    responses = {
        to_key(Requests.delete_label): (500, '{}')
    }

    returncode, requests = run(event_data, responses)

    assert_fail(returncode)


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/18255")
def test_close_active_without_label():
    event_data = default_data('closed')
    event_data['pull_request']['closed_at'] = '2019-07-05'

    returncode, requests = run(event_data)

    assert_success(returncode)
    assert [Requests.delete_ref_open] == requests


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/18255")
def test_open_with_label():
    event_data = default_data('opened')
    event_data['pull_request']['labels'].append(
        {'name': 'pull-request-has-preview'}
    )

    returncode, requests = run(event_data)

    assert_success(returncode)
    assert Requests.update_ref_open in requests
    assert Requests.update_ref_labeled in requests


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/18255")
def test_open_without_label_for_collaborator():
    event_data = default_data('opened')
    responses = {
        to_key(Requests.read_collaborator): (204, ''),
        to_key(Requests.get_ref_open): (404, '{}'),
        to_key(Requests.get_ref_labeled): (404, '{}'),
    }

    returncode, requests = run(event_data, responses)

    assert_success(returncode)
    assert Requests.create_label in requests
    assert Requests.create_ref_open in requests
    assert Requests.create_ref_labeled in requests


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/18255")
def test_open_without_label_for_non_collaborator():
    event_data = default_data('opened')
    responses = {
        ('GET', '/repos/test-org/test-repo/collaborators/rms'): (404, '{}')
    }

    returncode, requests = run(event_data, responses)

    assert_neutral(returncode)
    assert [Requests.read_collaborator] == requests


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/18255")
def test_add_unrelated_label():
    event_data = default_data('labeled')
    event_data['label'] = {'name': 'foobar'}
    event_data['pull_request']['labels'].append({'name': 'foobar'})

    returncode, requests = run(event_data)

    assert_neutral(returncode)
    assert len(requests) == 0


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/18255")
def test_add_active_label():
    event_data = default_data('labeled')
    event_data['label'] = {'name': 'pull-request-has-preview'}
    event_data['pull_request']['labels'].append(
        {'name': 'pull-request-has-preview'}
    )
    responses = {
        to_key(Requests.get_ref_open): (404, '{}'),
        to_key(Requests.get_ref_labeled): (404, '{}')
    }

    returncode, requests = run(event_data, responses)

    assert_success(returncode)
    assert Requests.create_ref_open not in requests
    assert Requests.create_ref_labeled in requests


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/18255")
def test_add_active_label_to_closed():
    event_data = default_data('labeled')
    event_data['pull_request']['closed_at'] = '2019-07-05'
    event_data['label'] = {'name': 'pull-request-has-preview'}
    event_data['pull_request']['labels'].append(
        {'name': 'pull-request-has-preview'}
    )
    responses = {
        to_key(Requests.get_ref_open): (404, '{}'),
        to_key(Requests.get_ref_labeled): (404, '{}')
    }

    returncode, requests = run(event_data, responses)

    assert_success(returncode)
    assert Requests.create_ref_open not in requests
    assert Requests.create_ref_labeled in requests


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/18255")
def test_remove_unrelated_label():
    event_data = default_data('unlabeled')
    event_data['label'] = {'name': 'foobar'}

    returncode, requests = run(event_data)

    assert_neutral(returncode)
    assert len(requests) == 0


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/18255")
def test_remove_active_label():
    event_data = default_data('unlabeled')
    event_data['label'] = {'name': 'pull-request-has-preview'}
    responses = {
        to_key(Requests.delete_ref_labeled): (204, '')
    }

    returncode, requests = run(event_data, responses)

    assert_success(returncode)
    assert Requests.delete_ref_labeled in requests
    assert Requests.delete_ref_open not in requests


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/18255")
def test_remove_active_label_from_closed():
    event_data = default_data('unlabeled')
    event_data['pull_request']['closed_at'] = '2019-07-05'
    event_data['label'] = {'name': 'pull-request-has-preview'}
    responses = {
        to_key(Requests.delete_ref_labeled): (204, '')
    }

    returncode, requests = run(event_data, responses)

    assert_success(returncode)
    assert Requests.delete_ref_labeled in requests
    assert Requests.delete_ref_open not in requests


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/18255")
def test_synchronize_without_label():
    event_data = default_data('synchronize')

    returncode, requests = run(event_data)

    assert_neutral(returncode)
    assert len(requests) == 0


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/18255")
def test_synchronize_with_label():
    event_data = default_data('synchronize')
    event_data['pull_request']['labels'].append(
        {'name': 'pull-request-has-preview'}
    )

    returncode, requests = run(event_data)

    assert_success(returncode)
    assert Requests.update_ref_open in requests
    assert Requests.update_ref_labeled in requests


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/18255")
def test_unrecognized_action():
    event_data = default_data('assigned')

    returncode, requests = run(event_data)

    assert_neutral(returncode)
    assert len(requests) == 0

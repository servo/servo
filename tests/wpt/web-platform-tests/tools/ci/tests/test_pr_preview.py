try:
    from BaseHTTPServer import BaseHTTPRequestHandler, HTTPServer
except ImportError:
    # Python 3 case
    from http.server import BaseHTTPRequestHandler, HTTPServer
import contextlib
import errno
import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
import threading

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
import pr_preview


TEST_HOST = 'localhost'


def same_members(a, b):
    if len(a) != len(b):
        return False
    a_copy = list(a)
    for elem in b:
        try:
            a_copy.remove(elem)
        except ValueError:
            return False

    return len(a_copy) == 0


# When these tests are executed in Windows, files in the temporary git
# repositories may be marked as "read only" at the moment they are intended to
# be deleted. The following handler for `shutil.rmtree` accounts for this by
# making the files writable and attempting to delete them a second time.
#
# Source:
# https://stackoverflow.com/questions/1213706/what-user-do-python-scripts-run-as-in-windows
def handle_remove_readonly(func, path, exc):
    excvalue = exc[1]
    candidates = (os.rmdir, os.remove, os.unlink)
    if func in candidates and excvalue.errno == errno.EACCES:
        os.chmod(path, stat.S_IRWXU | stat.S_IRWXG | stat.S_IRWXO)  # 0777
        func(path)
    else:
        raise


class MockHandler(BaseHTTPRequestHandler, object):
    def do_all(self):
        path = self.path.split('?')[0]
        request_body = None

        if 'Content-Length' in self.headers:
            request_body = self.rfile.read(
                int(self.headers['Content-Length'])
            ).decode('utf-8')

            if self.headers.get('Content-Type') == 'application/json':
                request_body = json.loads(request_body)

        for request, response in self.server.expected_traffic:
            if request[0] != self.command:
                continue
            if request[1] != path:
                continue
            body_matches = True
            for key in request[2]:
                body_matches &= request[2][key] == request_body.get(key)
            if not body_matches:
                continue
            break
        else:
            request = (self.command, path, request_body)
            response = (400, {})

        self.server.actual_traffic.append((request, response))
        self.send_response(response[0])
        self.end_headers()
        if self.server.reponse_body_is_json:
            self.wfile.write(json.dumps(response[1]).encode('utf-8'))
        else:
            self.wfile.write(response[1].encode('utf-8'))

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
    def __init__(self, address, expected_traffic, reponse_body_is_json=True):
        super(MockServer, self).__init__(address, MockHandler)
        self.expected_traffic = expected_traffic
        self.actual_traffic = []
        self.reponse_body_is_json = reponse_body_is_json

    def __enter__(self):
        threading.Thread(target=lambda: self.serve_forever()).start()
        return self

    def __exit__(self, *args):
        self.shutdown()


class Requests(object):
    get_rate = ('GET', '/rate_limit', {})
    ref_create_open = (
        'POST', '/repos/test-org/test-repo/git/refs', {'ref':'refs/prs-open/45'}
    )
    ref_create_trusted = (
        'POST',
        '/repos/test-org/test-repo/git/refs',
        {'ref':'refs/prs-trusted-for-preview/45'}
    )
    ref_get_open = (
        'GET', '/repos/test-org/test-repo/git/refs/prs-open/45', {}
    )
    ref_get_trusted = (
        'GET', '/repos/test-org/test-repo/git/refs/prs-trusted-for-preview/45', {}
    )
    ref_update_open = (
        'PATCH', '/repos/test-org/test-repo/git/refs/prs-open/45', {}
    )
    ref_update_trusted = (
        'PATCH', '/repos/test-org/test-repo/git/refs/prs-trusted-for-preview/45', {}
    )
    ref_delete_open = (
        'DELETE', '/repos/test-org/test-repo/git/refs/prs-open/45', {}
    )
    ref_delete_trusted = (
        'DELETE', '/repos/test-org/test-repo/git/refs/prs-trusted-for-preview/45', {}
    )
    deployment_get = ('GET', '/repos/test-org/test-repo/deployments', {})
    deployment_create = ('POST', '/repos/test-org/test-repo/deployments', {})
    deployment_status_create_pending = (
        'POST',
        '/repos/test-org/test-repo/deployments/24601/statuses',
        {'state':'pending'}
    )
    deployment_status_create_error = (
        'POST',
        '/repos/test-org/test-repo/deployments/24601/statuses',
        {'state':'error'}
    )
    deployment_status_create_success = (
        'POST',
        '/repos/test-org/test-repo/deployments/24601/statuses',
        {'state':'success'}
    )
    preview = ('GET', '/.git/worktrees/45/HEAD', {})


class Responses(object):
    no_limit = (200, {
        'resources': {
            'search': {
                'remaining': 100,
                'limit': 100
            },
            'core': {
                'remaining': 100,
                'limit': 100
            }
        }
    })


@contextlib.contextmanager
def temp_repo():
    original_dir = os.getcwd()
    directory = tempfile.mkdtemp()
    os.chdir(directory)

    try:
        subprocess.check_call(['git', 'init'], cwd=directory)
        # Explicitly create the default branch.
        subprocess.check_call(
            ['git', 'checkout', '-b', 'master'],
            cwd=directory
        )
        subprocess.check_call(
            ['git', 'config', 'user.name', 'example'],
            cwd=directory
        )
        subprocess.check_call(
            ['git', 'config', 'user.email', 'example@example.com'],
            cwd=directory
        )
        subprocess.check_call(
            ['git', 'commit', '--allow-empty', '-m', 'first'],
            cwd=directory
        )

        yield directory
    finally:
        os.chdir(original_dir)
        shutil.rmtree(
            directory, ignore_errors=False, onerror=handle_remove_readonly
        )

def update_mirror_refs(pull_request, expected_traffic):
    os.environ['GITHUB_TOKEN'] = 'c0ffee'

    github_server = MockServer((TEST_HOST, 0), expected_traffic)
    github_port = github_server.server_address[1]

    method_threw = False
    with temp_repo(), github_server:
        project = pr_preview.Project(
            'http://{}:{}'.format(TEST_HOST, github_port),
            'test-org/test-repo',
        )
        try:
            pr_preview.update_mirror_refs(project, pull_request)
        except pr_preview.GitHubRateLimitException:
            method_threw = True

    return (
        method_threw,
        github_server.actual_traffic,
    )


def deploy(pr_num, revision, expected_github_traffic, expected_preview_traffic):
    os.environ['GITHUB_TOKEN'] = 'c0ffee'

    github_server = MockServer((TEST_HOST, 0), expected_github_traffic)
    github_port = github_server.server_address[1]
    preview_server = MockServer((TEST_HOST, 0), expected_preview_traffic, reponse_body_is_json=False)
    preview_port = preview_server.server_address[1]

    method_threw = False
    with github_server, preview_server:
        project = pr_preview.Project(
            'http://{}:{}'.format(TEST_HOST, github_port),
            'test-org/test-repo',
        )
        target = 'http://{}:{}'.format(TEST_HOST, preview_port)
        pull_request = {'number': pr_num}
        timeout = 1
        try:
            pr_preview.deploy(project, target, pull_request, revision, timeout)
        except (pr_preview.GitHubRateLimitException, pr_preview.DeploymentFailedException):
            method_threw = True

    return (
        method_threw,
        github_server.actual_traffic,
        preview_server.actual_traffic
    )

def test_update_mirror_refs_fail_rate_limited():
    pull_request = {
        'number': 45,
        'head': {'sha': 'abc123'},
        'labels': [],
        'user': {'login': 'stephenmcgruer'},
        'author_association': 'COLLABORATOR',
        'closed_at': None,
    }
    expected_traffic = [
        (Requests.get_rate, (
            200,
            {
                'resources': {
                    'core': {
                        'remaining': 1,
                        'limit': 10
                    }
                }
            }
        ))
    ]

    method_threw, actual_traffic = update_mirror_refs(
        pull_request, expected_traffic
    )

    assert method_threw
    assert same_members(expected_traffic, actual_traffic)

def test_synchronize_ignore_closed():
    # No existing refs, but a closed PR event comes in. Nothing should happen.
    pull_request = {
        'number': 45,
        'head': {'sha': 'abc123'},
        'labels': [],
        'user': {'login': 'stephenmcgruer'},
        'author_association': 'COLLABORATOR',
        'closed_at': '2019-10-28',
    }
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_trusted, (404, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_open, (404, {})),
    ]

    method_threw, actual_traffic = update_mirror_refs(
        pull_request, expected_traffic
    )

    assert not method_threw
    assert same_members(expected_traffic, actual_traffic)

def test_update_mirror_refs_collaborator():
    pull_request = {
        'number': 45,
        'head': {'sha': 'abc123'},
        'labels': [],
        'user': {'login': 'stephenmcgruer'},
        'author_association': 'COLLABORATOR',
        'closed_at': None,
    }
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_trusted, (404, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_open, (404, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_create_open, (200, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_create_trusted, (200, {})),
    ]

    method_threw, actual_traffic, = update_mirror_refs(
        pull_request, expected_traffic
    )

    assert not method_threw
    assert same_members(expected_traffic, actual_traffic)

def test_update_mirror_refs_ignore_collaborator_bot():
    pull_request = {
        'number': 45,
        'head': {'sha': 'abc123'},
        'labels': [],
        'user': {'login': 'chromium-wpt-export-bot'},
        'author_association': 'COLLABORATOR',
        'closed_at': None,
    }
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_trusted, (404, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_open, (404, {})),
    ]

    method_threw, actual_traffic = update_mirror_refs(
        pull_request, expected_traffic
    )

    assert not method_threw
    assert same_members(expected_traffic, actual_traffic)

def test_update_mirror_refs_ignore_untrusted_contributor():
    pull_request = {
        'number': 45,
        'head': {'sha': 'abc123'},
        'labels': [],
        'user': {'login': 'stephenmcgruer'},
        'author_association': 'CONTRIBUTOR',
        'closed_at': None,
    }
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_trusted, (404, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_open, (404, {})),
    ]

    method_threw, actual_traffic = update_mirror_refs(
        pull_request, expected_traffic
    )

    assert not method_threw
    assert same_members(expected_traffic, actual_traffic)

def test_update_mirror_refs_trusted_contributor():
    pull_request = {
        'number': 45,
        'head': {'sha': 'abc123'},
        # user here is a contributor (untrusted), but the issue
        # has been labelled as safe.
        'labels': [{'name': 'safe for preview'}],
        'user': {'login': 'Hexcles'},
        'author_association': 'CONTRIBUTOR',
        'closed_at': None,
    }
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_trusted, (404, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_open, (404, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_create_open, (200, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_create_trusted, (200, {})),
    ]

    method_threw, actual_traffic = update_mirror_refs(
        pull_request, expected_traffic
    )

    assert not method_threw
    assert same_members(expected_traffic, actual_traffic)

def test_synchronize_sync_bot_with_label():
    pull_request = {
        'number': 45,
        'head': {'sha': 'abc123'},
        # user here is a bot which is normally not mirrored,
        # but the issue has been labelled as safe.
        'labels': [{'name': 'safe for preview'}],
        'user': {'login': 'chromium-wpt-export-bot'},
        'author_association': 'COLLABORATOR',
        'closed_at': None,
    }
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_trusted, (404, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_open, (404, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_create_open, (200, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_create_trusted, (200, {})),
    ]

    method_threw, actual_traffic = update_mirror_refs(
        pull_request, expected_traffic
    )

    assert not method_threw
    assert same_members(expected_traffic, actual_traffic)

def test_update_mirror_refs_update_collaborator():
    pull_request = {
        'number': 45,
        'head': {'sha': 'abc123'},
        'labels': [],
        'user': {'login': 'stephenmcgruer'},
        'author_association': 'COLLABORATOR',
        'closed_at': None,
    }
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_trusted, (
            200,
            {
                'object': {'sha': 'def234'},
            }
        )),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_open, (
            200,
            {
                'object': {'sha': 'def234'},
            }
        )),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_update_open, (200, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_update_trusted, (200, {})),
    ]

    method_threw, actual_traffic = update_mirror_refs(
        pull_request, expected_traffic
    )

    assert not method_threw
    assert same_members(expected_traffic, actual_traffic)

def test_synchronize_update_member():
    pull_request = {
        'number': 45,
        'head': {'sha': 'abc123'},
        'labels': [],
        'user': {'login': 'jgraham'},
        'author_association': 'MEMBER',
        'closed_at': None,
    }
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_trusted, (
            200,
            {
                'object': {'sha': 'def234'},
            }
        )),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_open, (
            200,
            {
                'object': {'sha': 'def234'},
            }
        )),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_update_open, (200, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_update_trusted, (200, {}))
    ]

    method_threw, actual_traffic = update_mirror_refs(
        pull_request, expected_traffic
    )

    assert not method_threw
    assert same_members(expected_traffic, actual_traffic)

def test_update_mirror_refs_delete_collaborator():
    pull_request = {
        'number': 45,
        'head': {'sha': 'abc123'},
        'labels': [],
        'user': {'login': 'stephenmcgruer'},
        'author_association': 'COLLABORATOR',
        'closed_at': 2019-10-30,
    }
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_trusted, (
            200,
            {
                'object': {'sha': 'def234'},
            }
        )),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_get_open, (
            200,
            {
                'object': {'sha': 'def234'},
            }
        )),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_delete_trusted, (204, None)),
        (Requests.get_rate, Responses.no_limit),
        (Requests.ref_delete_open, (204, None)),
    ]

    method_threw, actual_traffic = update_mirror_refs(
        pull_request, expected_traffic
    )

    assert not method_threw
    assert same_members(expected_traffic, actual_traffic)

def test_deploy_fail_rate_limited():
    expected_github_traffic = [
        (Requests.get_rate, (
            200,
            {
                'resources': {
                    'core': {
                        'remaining': 1,
                        'limit': 10
                    }
                }
            }
        ))
    ]
    expected_preview_traffic = []

    pr_num = 45
    revision = "abcdef123"
    method_threw, actual_github_traffic, actual_preview_traffic = deploy(
        pr_num, revision, expected_github_traffic, expected_preview_traffic
    )

    assert method_threw
    assert actual_github_traffic == expected_github_traffic
    assert actual_preview_traffic == expected_preview_traffic

def test_deploy_success():
    pr_num = 45
    revision = 'abcdef123'

    expected_github_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_get, (200, [])),
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_create, (200, {
            'id': 24601,
            'sha': revision,
            'environment': 'wpt-preview-45',
        })),
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_status_create_pending, (200, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_status_create_success, (200, {}))
    ]
    expected_preview_traffic = [
        (Requests.preview, (200, revision))
    ]

    method_threw, actual_github_traffic, actual_preview_traffic = deploy(
        pr_num, revision, expected_github_traffic, expected_preview_traffic
    )

    assert not method_threw
    assert actual_github_traffic == expected_github_traffic
    assert actual_preview_traffic == expected_preview_traffic

def test_deploy_timeout_missing():
    pr_num = 45
    revision = 'abcdef123'

    expected_github_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_get, (200, [])),
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_create, (200, {
            'id': 24601,
            'sha': revision,
            'environment': 'wpt-preview-45',
        })),
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_status_create_pending, (200, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_status_create_error, (200, {}))
    ]
    expected_preview_traffic = [
        (Requests.preview, (404, ""))
    ]

    method_threw, actual_github_traffic, actual_preview_traffic = deploy(
        pr_num, revision, expected_github_traffic, expected_preview_traffic
    )

    assert method_threw
    assert expected_github_traffic == actual_github_traffic
    ping_count = len(actual_preview_traffic)
    assert ping_count > 0
    assert actual_preview_traffic == expected_preview_traffic * ping_count

def test_deploy_timeout_wrong_revision():
    pr_num = 45
    revision = 'abcdef123'

    expected_github_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_get, (200, [])),
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_create, (200, {
            'id': 24601,
            'sha': revision,
            'environment': 'wpt-preview-45',
        })),
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_status_create_pending, (200, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_status_create_error, (200, {}))
    ]
    expected_preview_traffic = [
        # wptpr.live has the wrong revision deployed
        (Requests.preview, (200, 'ghijkl456'))
    ]

    method_threw, actual_github_traffic, actual_preview_traffic = deploy(
        pr_num, revision, expected_github_traffic, expected_preview_traffic
    )

    assert method_threw
    assert expected_github_traffic == actual_github_traffic
    ping_count = len(actual_preview_traffic)
    assert ping_count > 0
    assert actual_preview_traffic == expected_preview_traffic * ping_count

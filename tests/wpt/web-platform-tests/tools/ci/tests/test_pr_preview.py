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
import tempfile
import threading

subject = os.path.join(
    os.path.dirname(os.path.abspath(__file__)), '..', 'pr_preview.py'
)
test_host = 'localhost'


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
        self.wfile.write(json.dumps(response[1]).encode('utf-8'))

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
    def __init__(self, address, expected_traffic):
        super(MockServer, self).__init__(address, MockHandler)
        self.expected_traffic = expected_traffic
        self.actual_traffic = []

    def __enter__(self):
        threading.Thread(target=lambda: self.serve_forever()).start()
        return self

    def __exit__(self, *args):
        self.shutdown()


class Requests(object):
    get_rate = ('GET', '/rate_limit', {})
    search = ('GET', '/search/issues', {})
    pr_details = ('GET', '/repos/test-org/test-repo/pulls/23', {})
    ref_create_open = (
        'POST', '/repos/test-org/test-repo/git/refs', {'ref':'refs/prs-open/23'}
    )
    ref_create_trusted = (
        'POST',
        '/repos/test-org/test-repo/git/refs',
        {'ref':'refs/prs-trusted-for-preview/23'}
    )
    ref_update_open = (
        'PATCH', '/repos/test-org/test-repo/git/refs/prs-open/23', {}
    )
    ref_update_trusted = (
        'PATCH', '/repos/test-org/test-repo/git/refs/prs-trusted-for-preview/23', {}
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
    directory = tempfile.mkdtemp()

    try:
        subprocess.check_call(['git', 'init'], cwd=directory)
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
        shutil.rmtree(
            directory, ignore_errors=False, onerror=handle_remove_readonly
        )

def synchronize(expected_traffic, refs={}):
    env = {
        'DEPLOY_TOKEN': 'c0ffee'
    }
    env.update(os.environ)
    server = MockServer((test_host, 0), expected_traffic)
    test_port = server.server_address[1]
    remote_refs = {}

    with temp_repo() as local_repo, temp_repo() as remote_repo, server:
        subprocess.check_call(
            ['git', 'commit', '--allow-empty', '-m', 'first'],
            cwd=remote_repo
        )
        subprocess.check_call(
            ['git', 'commit', '--allow-empty', '-m', 'second'],
            cwd=remote_repo
        )
        subprocess.check_call(
            ['git', 'remote', 'add', 'origin', remote_repo], cwd=local_repo
        )

        for name, value in refs.items():
            subprocess.check_call(
                ['git', 'update-ref', name, value],
                cwd=remote_repo
            )

        child = subprocess.Popen(
            [
                'python',
                subject,
                '--host',
                'http://{}:{}'.format(test_host, test_port),
                '--github-project',
                'test-org/test-repo',
                'synchronize',
                '--window',
                '3000'
            ],
            cwd=local_repo,
            env=env
        )

        child.communicate()
        lines = subprocess.check_output(
            ['git', 'ls-remote', 'origin'], cwd=local_repo
        )
        for line in lines.decode('utf-8').strip().split('\n'):
            revision, ref = line.split()

            if not ref or ref in ('HEAD', 'refs/heads/master'):
                continue

            remote_refs[ref] = revision

    return child.returncode, server.actual_traffic, remote_refs


def detect(event, expected_github_traffic, expected_preview_traffic):
    env = {
        'DEPLOY_TOKEN': 'c0ffee'
    }
    env.update(os.environ)
    github_server = MockServer((test_host, 0), expected_github_traffic)
    github_port = github_server.server_address[1]
    preview_server = MockServer((test_host, 0), expected_preview_traffic)
    preview_port = preview_server.server_address[1]

    with temp_repo() as repo, github_server, preview_server:
        env['GITHUB_EVENT_PATH'] = repo + '/event.json'

        with open(env['GITHUB_EVENT_PATH'], 'w') as handle:
            handle.write(json.dumps(event))

        child = subprocess.Popen(
            [
                'python',
                subject,
                '--host',
                'http://{}:{}'.format(test_host, github_port),
                '--github-project',
                'test-org/test-repo',
                'detect',
                '--target',
                'http://{}:{}'.format(test_host, preview_port),
                '--timeout',
                '1'
            ],
            cwd=repo,
            env=env
        )
        child.communicate()

    return (
        child.returncode,
        github_server.actual_traffic,
        preview_server.actual_traffic
    )


def test_synchronize_zero_results():
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.search, (
            200,
            {
                'items': [],
                'incomplete_results': False
            }
        ))
    ]

    returncode, actual_traffic, remote_refs = synchronize(expected_traffic)

    assert returncode == 0
    assert same_members(expected_traffic, actual_traffic)

def test_synchronize_fail_search_throttled():
    expected_traffic = [
        (Requests.get_rate, (
            200,
            {
                'resources': {
                    'search': {
                        'remaining': 1,
                        'limit': 10
                    }
                }
            }
        ))
    ]

    returncode, actual_traffic, remote_refs = synchronize(expected_traffic)

    assert returncode != 0
    assert same_members(expected_traffic, actual_traffic)

def test_synchronize_fail_incomplete_results():
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.search, (
            200,
            {
                'items': [],
                'incomplete_results': True
            }
        ))
    ]

    returncode, actual_traffic, remove_refs = synchronize(expected_traffic)

    assert returncode != 0
    assert same_members(expected_traffic, actual_traffic)

def test_synchronize_ignore_closed():
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.search, (
            200,
            {
                'items': [
                    {
                        'number': 23,
                        'labels': [],
                        'closed_at': '2019-10-28',
                        'user': {'login': 'grace'},
                        'author_association': 'COLLABORATOR'
                    }
                ],
                'incomplete_results': False
            }
        ))
    ]

    returncode, actual_traffic, remote_refs = synchronize(expected_traffic)

    assert returncode == 0
    assert same_members(expected_traffic, actual_traffic)

def test_synchronize_sync_collaborator():
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.search, (
            200,
            {
                'items': [
                    {
                        'number': 23,
                        'labels': [],
                        'closed_at': None,
                        'user': {'login': 'grace'},
                        'author_association': 'COLLABORATOR'
                    }
                ],
                'incomplete_results': False
            }
        )),
        (Requests.pr_details, (200,
            {
                'head': {
                    'repo': {
                        'full_name': 'test-org/test-repo'
                    }
                }
            }
        )),
        (Requests.ref_create_open, (200, {})),
        (Requests.ref_create_trusted, (200, {})),
        (Requests.deployment_get, (200, {})),
        (Requests.deployment_create, (200, {}))
    ]

    returncode, actual_traffic, remote_refs = synchronize(expected_traffic)

    assert returncode == 0
    assert same_members(expected_traffic, actual_traffic)

def test_synchronize_ignore_collaborator_bot():
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.search, (
            200,
            {
                'items': [
                    {
                        'number': 23,
                        'labels': [],
                        'closed_at': None,
                        'user': {'login': 'chromium-wpt-export-bot'},
                        'author_association': 'COLLABORATOR'
                    }
                ],
                'incomplete_results': False
            }
        ))
    ]

    returncode, actual_traffic, remote_refs = synchronize(expected_traffic)

    assert returncode == 0
    assert same_members(expected_traffic, actual_traffic)

def test_synchronize_ignore_untrusted_contributor():
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.search, (
            200,
            {
                'items': [
                    {
                        'number': 23,
                        'labels': [],
                        'closed_at': None,
                        'user': {'login': 'grace'},
                        'author_association': 'CONTRIBUTOR'
                    }
                ],
                'incomplete_results': False
            }
        ))
    ]

    returncode, actual_traffic, remote_refs = synchronize(expected_traffic)

    assert returncode == 0
    assert same_members(expected_traffic, actual_traffic)

def test_synchronize_ignore_pull_request_from_fork():
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.search, (
            200,
            {
                'items': [
                    {
                        'number': 23,
                        'labels': [],
                        'closed_at': None,
                        'user': {'login': 'grace'},
                        'author_association': 'COLLABORATOR'
                    }
                ],
                'incomplete_results': False
            }
        )),
        (Requests.pr_details, (200,
            {
                'head': {
                    'repo': {
                        'full_name': 'some-other-org/test-repo'
                    }
                }
            }
        )),
    ]

    returncode, actual_traffic, remote_refs = synchronize(expected_traffic)

    assert returncode == 0
    assert same_members(expected_traffic, actual_traffic)

def test_synchronize_sync_trusted_contributor():
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.search, (
            200,
            {
                'items': [
                    {
                        'number': 23,
                        # user here is a contributor (untrusted), but the issue
                        # has been labelled as safe.
                        'labels': [{'name': 'safe for preview'}],
                        'closed_at': None,
                        'user': {'login': 'Hexcles'},
                        'author_association': 'CONTRIBUTOR'
                    }
                ],
                'incomplete_results': False
            }
        )),
        (Requests.pr_details, (200,
            {
                'head': {
                    'repo': {
                        'full_name': 'test-org/test-repo'
                    }
                }
            }
        )),
        (Requests.ref_create_open, (200, {})),
        (Requests.ref_create_trusted, (200, {})),
        (Requests.deployment_get, (200, [])),
        (Requests.deployment_create, (200, {}))
    ]

    returncode, actual_traffic, remote_refs = synchronize(expected_traffic)

    assert returncode == 0
    assert same_members(expected_traffic, actual_traffic)

def test_synchronize_sync_bot_with_label():
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.search, (
            200,
            {
                'items': [
                    {
                        'number': 23,
                        # user here is a bot which is normally not mirrored,
                        # but the issue has been labelled as safe.
                        'labels': [{'name': 'safe for preview'}],
                        'closed_at': None,
                        'user': {'login': 'chromium-wpt-export-bot'},
                        'author_association': 'COLLABORATOR'
                    }
                ],
                'incomplete_results': False
            }
        )),
        (Requests.pr_details, (200,
            {
                'head': {
                    'repo': {
                        'full_name': 'test-org/test-repo'
                    }
                }
            }
        )),
        (Requests.ref_create_open, (200, {})),
        (Requests.ref_create_trusted, (200, {})),
        (Requests.deployment_get, (200, [])),
        (Requests.deployment_create, (200, {}))
    ]

    returncode, actual_traffic, remote_refs = synchronize(expected_traffic)

    assert returncode == 0
    assert same_members(expected_traffic, actual_traffic)

def test_synchronize_update_collaborator():
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.search, (200,
            {
                'items': [
                    {
                        'number': 23,
                        'labels': [],
                        'closed_at': None,
                        'user': {'login': 'grace'},
                        'author_association': 'COLLABORATOR'
                    }
                ],
                'incomplete_results': False
            }
        )),
        (Requests.pr_details, (200,
            {
                'head': {
                    'repo': {
                        'full_name': 'test-org/test-repo'
                    }
                }
            }
        )),
        (Requests.deployment_get, (200, [])),
        (Requests.ref_update_open, (200, {})),
        (Requests.ref_update_trusted, (200, {})),
        (Requests.deployment_create, (200, {}))
    ]
    refs = {
        'refs/pull/23/head': 'HEAD',
        'refs/prs-open/23': 'HEAD~',
        'refs/prs-trusted-for-preview/23': 'HEAD~'
    }

    returncode, actual_traffic, remote_refs = synchronize(expected_traffic, refs)

    assert returncode == 0
    assert same_members(expected_traffic, actual_traffic)

def test_synchronize_update_member():
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.get_rate, Responses.no_limit),
        (Requests.search, (200,
            {
                'items': [
                    {
                        'number': 23,
                        'labels': [],
                        'closed_at': None,
                        'user': {'login': 'grace'},
                        'author_association': 'MEMBER'
                    }
                ],
                'incomplete_results': False
            }
        )),
        (Requests.pr_details, (200,
            {
                'head': {
                    'repo': {
                        'full_name': 'test-org/test-repo'
                    }
                }
            }
        )),
        (Requests.deployment_get, (200, [{'some': 'deployment'}])),
        (Requests.ref_update_open, (200, {})),
        (Requests.ref_update_trusted, (200, {}))
    ]
    refs = {
        'refs/pull/23/head': 'HEAD',
        'refs/prs-open/23': 'HEAD~',
        'refs/prs-trusted-for-preview/23': 'HEAD~'
    }

    returncode, actual_traffic, remote_refs = synchronize(expected_traffic, refs)

    assert returncode == 0
    assert same_members(expected_traffic, actual_traffic)

def test_synchronize_delete_collaborator():
    expected_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.search, (200,
            {
                'items': [
                    {
                        'number': 23,
                        'labels': [],
                        'closed_at': '2019-10-30',
                        'user': {'login': 'grace'},
                        'author_association': 'COLLABORATOR'
                    }
                ],
                'incomplete_results': False
            }
        ))
    ]
    refs = {
        'refs/pull/23/head': 'HEAD',
        'refs/prs-open/23': 'HEAD~',
        'refs/prs-trusted-for-preview/23': 'HEAD~'
    }

    returncode, actual_traffic, remote_refs = synchronize(expected_traffic, refs)

    assert returncode == 0
    assert same_members(expected_traffic, actual_traffic)
    assert list(remote_refs) == ['refs/pull/23/head']

def test_detect_ignore_unknown_env():
    expected_github_traffic = []
    expected_preview_traffic = []
    event = {
        'deployment': {
            'id': 24601,
            'environment': 'ghosts',
            'sha': '3232'
        }
    }

    returncode, actual_github_traffic, actual_preview_traffic = detect(
        event, expected_github_traffic, expected_preview_traffic
    )

    assert returncode == 0
    assert len(actual_github_traffic) == 0
    assert len(actual_preview_traffic) == 0

def test_detect_fail_search_throttled():
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
    event = {
        'deployment': {
            'id': 24601,
            'environment': 'wpt-preview-45',
            'sha': '3232'
        }
    }

    returncode, actual_github_traffic, actual_preview_traffic = detect(
        event, expected_github_traffic, expected_preview_traffic
    )

    assert returncode == 1
    assert actual_github_traffic == expected_github_traffic
    assert actual_preview_traffic == expected_preview_traffic

def test_detect_success():
    expected_github_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_status_create_pending, (200, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_status_create_success, (200, {}))
    ]
    expected_preview_traffic = [
        (Requests.preview, (200, 3232))
    ]
    event = {
        'deployment': {
            'id': 24601,
            'environment': 'wpt-preview-45',
            'sha': '3232'
        }
    }

    returncode, actual_github_traffic, actual_preview_traffic = detect(
        event, expected_github_traffic, expected_preview_traffic
    )

    assert returncode == 0
    assert actual_github_traffic == expected_github_traffic
    assert actual_preview_traffic == expected_preview_traffic

def test_detect_timeout_missing():
    expected_github_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_status_create_pending, (200, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_status_create_error, (200, {}))
    ]
    expected_preview_traffic = [
        (Requests.preview, (404, {}))
    ]
    event = {
        'deployment': {
            'id': 24601,
            'environment': 'wpt-preview-45',
            'sha': '3232'
        }
    }

    returncode, actual_github_traffic, actual_preview_traffic = detect(
        event, expected_github_traffic, expected_preview_traffic
    )

    assert returncode == 1
    assert expected_github_traffic == actual_github_traffic
    ping_count = len(actual_preview_traffic)
    assert ping_count > 0
    assert actual_preview_traffic == expected_preview_traffic * ping_count

def test_detect_timeout_wrong_revision():
    expected_github_traffic = [
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_status_create_pending, (200, {})),
        (Requests.get_rate, Responses.no_limit),
        (Requests.deployment_status_create_error, (200, {}))
    ]
    expected_preview_traffic = [
        (Requests.preview, (200, 1234))
    ]
    event = {
        'deployment': {
            'id': 24601,
            'environment': 'wpt-preview-45',
            'sha': '3232'
        }
    }

    returncode, actual_github_traffic, actual_preview_traffic = detect(
        event, expected_github_traffic, expected_preview_traffic
    )

    assert returncode == 1
    assert expected_github_traffic == actual_github_traffic
    ping_count = len(actual_preview_traffic)
    assert ping_count > 0
    assert actual_preview_traffic == expected_preview_traffic * ping_count

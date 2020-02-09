#!/usr/bin/env python

# The service provided by this script is not critical, but it shares a GitHub
# API request quota with critical services. For this reason, all requests to
# the GitHub API are preceded by a "guard" which verifies that the subsequent
# request will not deplete the shared quota.
#
# In effect, this script will fail rather than interfere with the operation of
# critical services.

import argparse
import json
import logging
import os
import subprocess
import time

import requests

# The ratio of "requests remaining" to "total request quota" below which this
# script should refuse to interact with the GitHub.com API
API_RATE_LIMIT_THRESHOLD = 0.2
# The GitHub Pull Request label which indicates that a Pull Request is expected
# to be actively mirrored by the preview server
LABEL = 'safe for preview'
# The number of seconds to wait between attempts to verify that a submission
# preview is available on the Pull Request preview server
POLLING_PERIOD = 5
# Pull Requests from authors with the following associations to the project
# should automatically receive previews
#
# https://developer.github.com/v4/enum/commentauthorassociation/ (equivalent
# documentation for the REST API was not available at the time of writing)
TRUSTED_AUTHOR_ASSOCIATIONS = ('COLLABORATOR', 'MEMBER', 'OWNER')
# These GitHub accounts are not associated with individuals, and the Pull
# Requests they submit rarely require a preview.
AUTOMATION_GITHUB_USERS = (
    'autofoolip', 'chromium-wpt-export-bot', 'moz-wptsync-bot',
    'servo-wpt-sync'
)
DEPLOYMENT_PREFIX = 'wpt-preview-'

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def gh_request(method_name, url, body=None, media_type=None):
    github_token = os.environ['DEPLOY_TOKEN']

    kwargs = {
        'headers': {
            'Authorization': 'token {}'.format(github_token),
            'Accept': media_type or 'application/vnd.github.v3+json'
        }
    }
    method = getattr(requests, method_name.lower())

    if body is not None:
        kwargs['json'] = body

    logger.info('Issuing request: %s %s', method_name.upper(), url)

    resp = method(url, **kwargs)

    logger.info('Response status code: %s', resp.status_code)

    resp.raise_for_status()

    return resp.json()

def guard(resource):
    '''Decorate a `Project` instance method which interacts with the GitHub
    API, ensuring that the subsequent request will not deplete the relevant
    allowance. This verification does not itself influence rate limiting:

    > Accessing this endpoint does not count against your REST API rate limit.

    https://developer.github.com/v3/rate_limit/
    '''
    def guard_decorator(func):
        def wrapped(self, *args, **kwargs):
            limits = gh_request('GET', '{}/rate_limit'.format(self._host))

            values = limits['resources'].get(resource)

            remaining = values['remaining']
            limit = values['limit']

            logger.info(
                'Limit for "%s" resource: %s/%s', resource, remaining, limit
            )

            if limit and float(remaining) / limit < API_RATE_LIMIT_THRESHOLD:
                raise Exception(
                    'Exiting to avoid GitHub.com API request throttling.'
                )

            return func(self, *args, **kwargs)
        return wrapped
    return guard_decorator

class Project(object):
    def __init__(self, host, github_project):
        self._host = host
        self._github_project = github_project

    @guard('search')
    def get_pull_requests(self, updated_since):
        window_start = time.strftime('%Y-%m-%dT%H:%M:%SZ', updated_since)
        url = '{}/search/issues?q=repo:{}+is:pr+updated:>{}'.format(
            self._host, self._github_project, window_start
        )

        logger.info(
            'Searching for Pull Requests updated since %s', window_start
        )

        data = gh_request('GET', url)

        logger.info('Found %d Pull Requests', len(data['items']))

        if data['incomplete_results']:
            raise Exception('Incomplete results')

        return data['items']

    @guard('core')
    def create_ref(self, refspec, revision):
        url = '{}/repos/{}/git/refs'.format(self._host, self._github_project)

        logger.info('Creating ref "%s" (%s)', refspec, revision)

        gh_request('POST', url, {
            'ref': 'refs/{}'.format(refspec),
            'sha': revision
        })

    @guard('core')
    def update_ref(self, refspec, revision):
        url = '{}/repos/{}/git/refs/{}'.format(
            self._host, self._github_project, refspec
        )

        logger.info('Updating ref "%s" (%s)', refspec, revision)

        gh_request('PATCH', url, {'sha': revision})

    @guard('core')
    def create_deployment(self, pull_request, revision):
        url = '{}/repos/{}/deployments'.format(
            self._host, self._github_project
        )
        # The Pull Request preview system only exposes one Deployment for a
        # given Pull Request. Identifying the Deployment by the Pull Request
        # number ensures that GitHub.com automatically responds to new
        # Deployments by designating prior Deployments as "inactive"
        environment = DEPLOYMENT_PREFIX + str(pull_request['number'])

        logger.info('Creating Deployment "%s" for "%s"', environment, revision)

        return gh_request('POST', url, {
            'ref': revision,
            'environment': environment,
            'auto_merge': False,
            # Pull Request previews are created regardless of GitHub Commit
            # Status Checks, so Status Checks should be ignored when creating
            # GitHub Deployments.
            'required_contexts': []
        }, 'application/vnd.github.ant-man-preview+json')

    @guard('core')
    def get_deployment(self, revision):
        url = '{}/repos/{}/deployments?sha={}'.format(
            self._host, self._github_project, revision
        )

        deployments = gh_request('GET', url)

        return deployments.pop() if len(deployments) else None

    @guard('core')
    def update_deployment(self, target, deployment, state, description=''):
        if state in ('pending', 'success'):
            environment_url = '{}/submissions/{}'.format(
                target, deployment['environment']
            )
        else:
            environment_url = None
        url = '{}/repos/{}/deployments/{}/statuses'.format(
            self._host, self._github_project, deployment['id']
        )

        gh_request('POST', url, {
            'state': state,
            'description': description,
            'environment_url': environment_url
        }, 'application/vnd.github.ant-man-preview+json')

class Remote(object):
    def __init__(self, github_project):
        # The repository in the GitHub Actions environment is configured with
        # a remote whose URL uses unauthenticated HTTPS, making it unsuitable
        # for pushing changes.
        self._token = os.environ['DEPLOY_TOKEN']

    def get_revision(self, refspec):
        output = subprocess.check_output([
            'git',
            '-c',
            'credential.username={}'.format(self._token),
            '-c',
            'core.askPass=true',
            'ls-remote',
            'origin',
            'refs/{}'.format(refspec)
        ])

        if not output:
            return None

        return output.decode('utf-8').split()[0]

    def delete_ref(self, refspec):
        full_ref = 'refs/{}'.format(refspec)

        logger.info('Deleting ref "%s"', refspec)

        subprocess.check_call([
            'git',
            '-c',
            'credential.username={}'.format(self._token),
            '-c',
            'core.askPass=true',
            'push',
            'origin',
            '--delete',
            full_ref
        ])

def is_open(pull_request):
    return not pull_request['closed_at']

def has_mirroring_label(pull_request):
    for label in pull_request['labels']:
        if label['name'] == LABEL:
            return True

    return False

def should_be_mirrored(pull_request):
    return (
        is_open(pull_request) and
        pull_request['user']['login'] not in AUTOMATION_GITHUB_USERS and (
            pull_request['author_association'] in TRUSTED_AUTHOR_ASSOCIATIONS or
            has_mirroring_label(pull_request)
        )
    )

def is_deployed(host, deployment):
    worktree_name = deployment['environment'][len(DEPLOYMENT_PREFIX):]
    response = requests.get(
        '{}/.git/worktrees/{}/HEAD'.format(host, worktree_name)
    )

    if response.status_code != 200:
        return False

    return response.text.strip() == deployment['sha']

def synchronize(host, github_project, window):
    '''Inspect all Pull Requests which have been modified in a given window of
    time. Add or remove the "preview" label and update or delete the relevant
    git refs according to the status of each Pull Request.'''

    project = Project(host, github_project)
    remote = Remote(github_project)

    pull_requests = project.get_pull_requests(
        time.gmtime(time.time() - window)
    )

    for pull_request in pull_requests:
        logger.info('Processing Pull Request #%(number)d', pull_request)

        refspec_trusted = 'prs-trusted-for-preview/{number}'.format(
            **pull_request
        )
        refspec_open = 'prs-open/{number}'.format(**pull_request)
        revision_latest = remote.get_revision(
            'pull/{number}/head'.format(**pull_request)
        )
        revision_trusted = remote.get_revision(refspec_trusted)
        revision_open = remote.get_revision(refspec_open)

        if should_be_mirrored(pull_request):
            logger.info('Pull Request should be mirrored')

            if revision_trusted is None:
                project.create_ref(refspec_trusted, revision_latest)
            elif revision_trusted != revision_latest:
                project.update_ref(refspec_trusted, revision_latest)

            if revision_open is None:
                project.create_ref(refspec_open, revision_latest)
            elif revision_open != revision_latest:
                project.update_ref(refspec_open, revision_latest)

            if project.get_deployment(revision_latest) is None:
                project.create_deployment(
                    pull_request, revision_latest
                )
        else:
            logger.info('Pull Request should not be mirrored')

            if not has_mirroring_label(pull_request) and revision_trusted is not None:
                remote.delete_ref(refspec_trusted)

            if revision_open is not None and not is_open(pull_request):
                remote.delete_ref(refspec_open)

def detect(host, github_project, target, timeout):
    '''Manage the status of a GitHub Deployment by polling the Pull Request
    preview website until the Deployment is complete or a timeout is
    reached.'''

    project = Project(host, github_project)

    with open(os.environ['GITHUB_EVENT_PATH']) as handle:
        data = json.loads(handle.read())

    logger.info('Event data: %s', json.dumps(data, indent=2))

    deployment = data['deployment']

    if not deployment['environment'].startswith(DEPLOYMENT_PREFIX):
        logger.info(
            'Deployment environment "%s" is unrecognized. Exiting.',
            deployment['environment']
        )
        return

    message = 'Waiting up to {} seconds for Deployment {} to be available on {}'.format(
        timeout, deployment['environment'], target
    )
    logger.info(message)
    project.update_deployment(target, deployment, 'pending', message)

    start = time.time()

    while not is_deployed(target, deployment):
        if time.time() - start > timeout:
            message = 'Deployment did not become available after {} seconds'.format(timeout)
            project.update_deployment(target, deployment, 'error', message)
            raise Exception(message)

        time.sleep(POLLING_PERIOD)

    result = project.update_deployment(target, deployment, 'success')
    logger.info(json.dumps(result, indent=2))

if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        description='''Synchronize the state of a GitHub.com project with the
            underlying git repository in order to support a externally-hosted
            Pull Request preview system. Communicate the state of that system
            via GitHub Deployments associated with each Pull Request.'''
    )
    parser.add_argument(
        '--host', required=True, help='the location of the GitHub API server'
    )
    parser.add_argument(
        '--github-project',
        required=True,
        help='''the GitHub organization and GitHub project name, separated by
            a forward slash (e.g. "web-platform-tests/wpt")'''
    )
    subparsers = parser.add_subparsers(title='subcommands')

    parser_sync = subparsers.add_parser(
        'synchronize', help=synchronize.__doc__
    )
    parser_sync.add_argument(
        '--window',
        type=int,
        required=True,
        help='''the number of seconds prior to the current moment within which
            to search for GitHub Pull Requests. Any Pull Requests updated in
            this time frame will be considered for synchronization.'''
    )
    parser_sync.set_defaults(func=synchronize)

    parser_detect = subparsers.add_parser('detect', help=detect.__doc__)
    parser_detect.add_argument(
        '--target',
        required=True,
        help='''the URL of the website to which submission previews are
            expected to become available'''
    )
    parser_detect.add_argument(
        '--timeout',
        type=int,
        required=True,
        help='''the number of seconds to wait for a submission preview to
            become available before reporting a GitHub Deployment failure'''
    )
    parser_detect.set_defaults(func=detect)

    values = dict(vars(parser.parse_args()))
    values.pop('func')(**values)

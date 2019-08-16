# wpt-submissions.live is a public deployment of WPT, maintained in an external
# repository. It automatically fetches and deploys all refs in the WPT
# repository which match a certain pattern. This behavior is intended to be
# used for pull requests so that reviewers can preview changes without running
# the WPT server locally.
#
# This script facilitates the service by maintaining the git refs. It creates
# and updates refs in response to GitHub events. It does this automatically for
# pull requests from GitHub users who have "collaborator" access permissions to
# the WPT repository. It also does this for any pull requests which bear the
# `pull-request-has-preview` label. Collaborators can add or remove this label
# to enable or disable the preview for submissions from non-collaborators.
#
# Although the script relies on a secret access token, it is *not* limited to
# use for pull requests from trusted collaborators due to the way GitHub
# Actions are executed:
#
# > # Pull request events for forked repositories
# >
# > [...]
# >
# > ## Pull request with base and head branches in different repositories
# >
# > The base repository receives a `pull_request` event where the SHA is the
# > latest commit of base branch and ref is the base branch.
#
# https://developer.github.com/actions/managing-workflows/workflow-configuration-options/#pull-request-events-for-forked-repositories
import json
import logging
import os
import sys

import requests

active_label = 'pull-request-has-preview'

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class Status(object):
    SUCCESS = 0
    FAIL = 1
    NEUTRAL = 0


def request(url, method_name, data=None, json_data=None, ignore_body=False):
    github_token = os.environ.get('GITHUB_TOKEN')

    kwargs = {
        'headers': {
            'Authorization': 'token {}'.format(github_token),
            'Accept': 'application/vnd.github.machine-man-preview+json'
        }
    }
    method = getattr(requests, method_name)

    logger.info('Issuing request: {} {}'.format(method_name.upper(), url))
    if json_data is not None or data is not None:
        kwargs['json'] = json_data
        kwargs['data'] = data

    resp = method(url, **kwargs)

    resp.raise_for_status()

    if not ignore_body:
        return resp.json()


def resource_exists(url):
    try:
        request(url, 'get', ignore_body=True)
    except requests.HTTPError as exception:
        if exception.response.status_code == 404:
            return False
        raise

    return True


class GitHub(object):
    def __init__(self, api_root, owner, repo):
        self.api_root = api_root
        self.owner = owner
        self.repo = repo

    def is_collaborator(self, login):
        return resource_exists(
            '{}/repos/{}/{}/collaborators/{}'.format(
                self.api_root, self.owner, self.repo, login
            )
        )

    def ref_exists(self, ref):
        return resource_exists(
            '{}/repos/{}/{}/git/refs/{}'.format(
                self.api_root, self.owner, self.repo, ref
            )
        )

    def create_ref(self, ref, sha):
        data = {
            'ref': 'refs/{}'.format(ref),
            'sha': sha
        }
        url = '{}/repos/{}/{}/git/refs'.format(
            self.api_root, self.owner, self.repo
        )

        logger.info('Creating ref "{}" as {}'.format(ref, sha))

        request(url, 'post', json_data=data)

    def update_ref(self, ref, sha):
        data = {
            'force': True,
            'sha': sha
        }
        url = '{}/repos/{}/{}/git/refs/{}'.format(
            self.api_root, self.owner, self.repo, ref
        )

        logger.info('Updating ref "{}" as {}'.format(ref, sha))

        request(url, 'patch', json_data=data)

    def delete_ref(self, ref):
        url = '{}/repos/{}/{}/git/refs/{}'.format(
            self.api_root, self.owner, self.repo, ref
        )

        logger.info('Deleting ref "{}"'.format(ref))

        try:
            request(url, 'delete', ignore_body=True)
        except requests.HTTPError as exception:
            if exception.response.status_code != 404:
                raise

            logger.info(
                'Attempted to delete non-existent ref: {}'.format(ref)
            )

    def set_ref(self, ref, sha):
        if self.ref_exists(ref):
            self.update_ref(ref, sha)
        else:
            self.create_ref(ref, sha)

    def add_label(self, pr_number, label_name):
        data = {
            'labels': [label_name]
        }
        url = '{}/repos/{}/{}/issues/{}/labels'.format(
            self.api_root, self.owner, self.repo, pr_number
        )

        logger.info('Adding label')

        request(url, 'post', json_data=data)

    def remove_label(self, pr_number, label_name):
        url = '{}/repos/{}/{}/issues/{}/labels/{}'.format(
            self.api_root, self.owner, self.repo, pr_number, label_name
        )

        logger.info('Removing label')

        try:
            request(url, 'delete')
        except requests.HTTPError as exception:
            if exception.response.status_code != 404:
                raise

            logger.info(
                'Attempted to remove non-existent label: {}'.format(label_name)
            )


def main(api_root):
    with open(os.environ['GITHUB_EVENT_PATH']) as handle:
        event = json.load(handle)
        logger.info(json.dumps(event, indent=2))

    if 'pull_request' not in event:
        logger.info('Unexpected event data')
        return Status.FAIL

    owner, repo = os.environ['GITHUB_REPOSITORY'].split('/', 1)
    github = GitHub(api_root, owner, repo)
    action = event['action']
    pr_number = event['pull_request']['number']
    ref_open = 'prs-open/gh-{}'.format(pr_number)
    ref_labeled = 'prs-labeled-for-preview/gh-{}'.format(pr_number)
    sha = event['pull_request']['head']['sha']
    login = event['pull_request']['user']['login']
    has_label = any([
        label['name'] == active_label
        for label in event['pull_request']['labels']
    ])
    target_label = event.get('label', {}).get('name')

    if action == 'closed':
        if has_label:
            github.remove_label(pr_number, active_label)

        # Removing a label from a GitHub Action will not trigger another
        # Workflow, so the corresponding ref must be deleted while processing
        # the "closed" action.
        #
        # > An action can't trigger other workflows. For example, a push,
        # > deployment, or any task performed within an action with the
        # > provided `GITHUB_TOKEN` will not trigger a workflow listening on
        # > push, deploy, or any other supported action triggers.
        #
        # https://developer.github.com/actions/managing-workflows/workflow-configuration-options/
        github.delete_ref(ref_open)

        return Status.SUCCESS
    elif action in ('opened', 'reopened') and has_label:
        github.set_ref(ref_open, sha)
        github.set_ref(ref_labeled, sha)
    elif action in ('opened', 'reopened') and github.is_collaborator(login):
        github.add_label(pr_number, active_label)

        # Removing a label from a GitHub Action will not trigger another
        # Workflow, so the corresponding ref must be created while processing
        # the "opened" and "reopened" actions.
        #
        # > An action can't trigger other workflows. For example, a push,
        # > deployment, or any task performed within an action with the
        # > provided `GITHUB_TOKEN` will not trigger a workflow listening on
        # > push, deploy, or any other supported action triggers.
        #
        # https://developer.github.com/actions/managing-workflows/workflow-configuration-options/
        github.set_ref(ref_open, sha)
        github.set_ref(ref_labeled, sha)
    elif action == 'labeled' and target_label == active_label:
        github.set_ref(ref_labeled, sha)
    elif action == 'unlabeled' and target_label == active_label:
        github.delete_ref(ref_labeled)
    elif action == 'synchronize' and has_label:
        github.set_ref(ref_open, sha)
        github.set_ref(ref_labeled, sha)
    else:
        return Status.NEUTRAL

    return Status.SUCCESS


if __name__ == '__main__':
    code = main(sys.argv[1])
    assert isinstance(code, int)
    sys.exit(code)

# mypy: allow-untyped-defs

import argparse
import logging
import os

import taskcluster

from .github_checks_output import get_gh_checks_outputter


logging.basicConfig()
logger = logging.getLogger()


def check_task_statuses(task_ids, github_checks_outputter):
    """Verifies whether a set of Taskcluster tasks completed successfully or not.

    Returns 0 if all tasks passed completed successfully, 1 otherwise."""

    queue = taskcluster.Queue({'rootUrl': os.environ['TASKCLUSTER_ROOT_URL']})
    failed_tasks = []
    for task in task_ids:
        status = queue.status(task)
        state = status['status']['state']
        if state == 'failed' or state == 'exception':
            logger.error(f'Task {task} failed with state "{state}"')
            failed_tasks.append(status)
        elif state != 'completed':
            logger.error(f'Task {task} had unexpected state "{state}"')
            failed_tasks.append(status)

    if failed_tasks and github_checks_outputter:
        github_checks_outputter.output('Failed tasks:')
        for task in failed_tasks:
            # We need to make an additional call to get the task name.
            task_id = task['status']['taskId']
            task_name = queue.task(task_id)['metadata']['name']
            github_checks_outputter.output('* `{}` failed with status `{}`'.format(task_name, task['status']['state']))
    else:
        logger.info('All tasks completed successfully')
        if github_checks_outputter:
            github_checks_outputter.output('All tasks completed successfully')
    return 1 if failed_tasks else 0


def get_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("--github-checks-text-file", type=str,
            help="Path to GitHub checks output file for Taskcluster runs")
    parser.add_argument("tasks", nargs="+",
            help="A set of Taskcluster task ids to verify the state of.")
    return parser


def run(venv, **kwargs):
    github_checks_outputter = get_gh_checks_outputter(kwargs["github_checks_text_file"])

    if github_checks_outputter:
        github_checks_outputter.output(
            "This check acts as a 'sink' for all other Taskcluster-based checks. "
            "A failure here means that some other check has failed, which is the "
            "real blocker.\n"
        )
    return check_task_statuses(kwargs['tasks'], github_checks_outputter)

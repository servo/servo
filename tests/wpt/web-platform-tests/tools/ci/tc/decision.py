import argparse
import json
import logging
import os
import re
import subprocess
from collections import OrderedDict

import taskcluster
from six import iteritems, itervalues

from . import taskgraph


here = os.path.abspath(os.path.dirname(__file__))


logging.basicConfig()
logger = logging.getLogger()


def get_triggers(event):
    # Set some variables that we use to get the commits on the current branch
    ref_prefix = "refs/heads/"
    is_pr = "pull_request" in event
    branch = None
    if not is_pr and "ref" in event:
        branch = event["ref"]
        if branch.startswith(ref_prefix):
            branch = branch[len(ref_prefix):]

    return is_pr, branch


def fetch_event_data(queue):
    try:
        task_id = os.environ["TASK_ID"]
    except KeyError:
        logger.warning("Missing TASK_ID environment variable")
        # For example under local testing
        return None

    task_data = queue.task(task_id)

    return task_data.get("extra", {}).get("github_event")


def filter_triggers(event, all_tasks):
    is_pr, branch = get_triggers(event)
    triggered = {}
    for name, task in iteritems(all_tasks):
        if "trigger" in task:
            if is_pr and "pull-request" in task["trigger"]:
                triggered[name] = task
            elif branch is not None and "branch" in task["trigger"]:
                for trigger_branch in task["trigger"]["branch"]:
                    if (trigger_branch == branch or
                        trigger_branch.endswith("*") and branch.startswith(trigger_branch[:-1])):
                        triggered[name] = task
    logger.info("Triggers match tasks:\n * %s" % "\n * ".join(triggered.keys()))
    return triggered


def get_run_jobs(event):
    from tools.ci import jobs
    revish = "%s..%s" % (event["pull_request"]["base"]["sha"]
                         if "pull_request" in event
                         else event["before"],
                         event["pull_request"]["head"]["sha"]
                         if "pull_request" in event
                         else event["after"])
    logger.info("Looking for changes in range %s" % revish)
    paths = jobs.get_paths(revish=revish)
    logger.info("Found changes in paths:%s" % "\n".join(paths))
    path_jobs = jobs.get_jobs(paths)
    all_jobs = path_jobs | get_extra_jobs(event)
    logger.info("Including jobs:\n * %s" % "\n * ".join(all_jobs))
    return all_jobs


def get_extra_jobs(event):
    body = None
    jobs = set()
    if "commits" in event and event["commits"]:
        body = event["commits"][0]["message"]
    elif "pull_request" in event:
        body = event["pull_request"]["body"]

    if not body:
        return jobs

    regexp = re.compile(r"\s*tc-jobs:(.*)$")

    for line in body.splitlines():
        m = regexp.match(line)
        if m:
            items = m.group(1)
            for item in items.split(","):
                jobs.add(item.strip())
            break
    return jobs


def filter_schedule_if(event, tasks):
    scheduled = {}
    run_jobs = None
    for name, task in iteritems(tasks):
        if "schedule-if" in task:
            if "run-job" in task["schedule-if"]:
                if run_jobs is None:
                    run_jobs = get_run_jobs(event)
                if "all" in run_jobs or any(item in run_jobs for item in task["schedule-if"]["run-job"]):
                    scheduled[name] = task
        else:
            scheduled[name] = task
    logger.info("Scheduling rules match tasks:\n * %s" % "\n * ".join(scheduled.keys()))
    return scheduled


def get_fetch_rev(event):
    is_pr, _ = get_triggers(event)
    if is_pr:
        # Try to get the actual rev so that all non-decision tasks are pinned to that
        rv = ["refs/pull/%s/merge" % event["pull_request"]["number"]]
        # For every PR GitHub maintains a 'head' branch with commits from the
        # PR, and a 'merge' branch containing a merge commit between the base
        # branch and the PR.
        for ref_type in ["head", "merge"]:
            ref = "refs/pull/%s/%s" % (event["pull_request"]["number"], ref_type)
            sha = None
            try:
                output = subprocess.check_output(["git", "ls-remote", "origin", ref])
            except subprocess.CalledProcessError:
                import traceback
                logger.error(traceback.format_exc())
                logger.error("Failed to get commit sha1 for %s" % ref)
            else:
                if not output:
                    logger.error("Failed to get commit for %s" % ref)
                else:
                    sha = output.split()[0]
            rv.append(sha)
        rv = tuple(rv)
    else:
        # For a branch push we have a ref and a head but no merge SHA
        rv = (event["ref"], event["after"], None)
    assert len(rv) == 3
    return rv


def build_full_command(event, task):
    fetch_ref, head_sha, merge_sha = get_fetch_rev(event)
    cmd_args = {
        "task_name": task["name"],
        "repo_url": event["repository"]["clone_url"],
        "fetch_ref": fetch_ref,
        "task_cmd": task["command"],
        "install_str": "",
    }

    options = task.get("options", {})
    options_args = []
    options_args.append("--ref=%s" % fetch_ref)
    if head_sha is not None:
        options_args.append("--head-rev=%s" % head_sha)
    if merge_sha is not None:
        options_args.append("--merge-rev=%s" % merge_sha)
    if options.get("oom-killer"):
        options_args.append("--oom-killer")
    if options.get("xvfb"):
        options_args.append("--xvfb")
    if not options.get("hosts"):
        options_args.append("--no-hosts")
    else:
        options_args.append("--hosts")
    # Check out the expected SHA unless it is overridden (e.g. to base_head).
    if options.get("checkout"):
        options_args.append("--checkout=%s" % options["checkout"])
    for browser in options.get("browser", []):
        options_args.append("--browser=%s" % browser)
    if options.get("channel"):
        options_args.append("--channel=%s" % options["channel"])
    if options.get("install-certificates"):
        options_args.append("--install-certificates")

    cmd_args["options_str"] = " ".join(str(item) for item in options_args)

    install_packages = task.get("install")
    if install_packages:
        install_items = ["apt update -qqy"]
        install_items.extend("apt install -qqy %s" % item
                             for item in install_packages)
        cmd_args["install_str"] = "\n".join("sudo %s;" % item for item in install_items)

    return ["/bin/bash",
            "--login",
            "-xc",
            """
~/start.sh \
  %(repo_url)s \
  %(fetch_ref)s;
%(install_str)s
cd web-platform-tests;
./tools/ci/run_tc.py %(options_str)s -- %(task_cmd)s;
""" % cmd_args]


def get_owner(event):
    if "pusher" in event:
        pusher = event.get("pusher", {}).get("email", "")
        if pusher and "@" in pusher:
            return pusher
    return "web-platform-tests@users.noreply.github.com"


def create_tc_task(event, task, taskgroup_id, depends_on_ids):
    command = build_full_command(event, task)
    task_id = taskcluster.slugId()
    task_data = {
        "taskGroupId": taskgroup_id,
        "created": taskcluster.fromNowJSON(""),
        "deadline": taskcluster.fromNowJSON(task["deadline"]),
        "provisionerId": task["provisionerId"],
        "schedulerId": task["schedulerId"],
        "workerType": task["workerType"],
        "metadata": {
            "name": task["name"],
            "description": task.get("description", ""),
            "owner": get_owner(event),
            "source": event["repository"]["clone_url"]
        },
        "payload": {
            "artifacts": task.get("artifacts"),
            "command": command,
            "image": task.get("image"),
            "maxRunTime": task.get("maxRunTime"),
            "env": task.get("env", {}),
        },
        "extra": {
            "github_event": json.dumps(event)
        }
    }
    if depends_on_ids:
        task_data["dependencies"] = depends_on_ids
        task_data["requires"] = "all-completed"
    return task_id, task_data


def build_task_graph(event, all_tasks, tasks):
    task_id_map = OrderedDict()
    taskgroup_id = os.environ.get("TASK_ID", taskcluster.slugId())

    def add_task(task_name, task):
        depends_on_ids = []
        if "depends-on" in task:
            for depends_name in task["depends-on"]:
                if depends_name not in task_id_map:
                    add_task(depends_name,
                             all_tasks[depends_name])
                depends_on_ids.append(task_id_map[depends_name][0])
        task_id, task_data = create_tc_task(event, task, taskgroup_id, depends_on_ids)
        task_id_map[task_name] = (task_id, task_data)

    for task_name, task in iteritems(tasks):
        add_task(task_name, task)

    return task_id_map


def create_tasks(queue, task_id_map):
    for (task_id, task_data) in itervalues(task_id_map):
        queue.createTask(task_id, task_data)


def get_event(queue, event_path):
    if event_path is not None:
        try:
            with open(event_path) as f:
                event_str = f.read()
        except IOError:
            logger.error("Missing event file at path %s" % event_path)
            raise
    elif "TASK_EVENT" in os.environ:
        event_str = os.environ["TASK_EVENT"]
    else:
        event_str = fetch_event_data(queue)
    if not event_str:
        raise ValueError("Can't find GitHub event definition; for local testing pass --event-path")
    try:
        return json.loads(event_str)
    except ValueError:
        logger.error("Event was not valid JSON")
        raise


def decide(event):
    all_tasks = taskgraph.load_tasks_from_path(os.path.join(here, "tasks", "test.yml"))

    triggered_tasks = filter_triggers(event, all_tasks)
    scheduled_tasks = filter_schedule_if(event, triggered_tasks)

    logger.info("UNSCHEDULED TASKS:\n  %s" % "\n  ".join(sorted(set(all_tasks.keys()) -
                                                            set(scheduled_tasks.keys()))))
    logger.info("SCHEDULED TASKS:\n  %s" % "\n  ".join(sorted(scheduled_tasks.keys())))

    task_id_map = build_task_graph(event, all_tasks, scheduled_tasks)
    return task_id_map


def get_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("--event-path",
                        help="Path to file containing serialized GitHub event")
    parser.add_argument("--dry-run", action="store_true",
                        help="Don't actually create the tasks, just output the tasks that "
                        "would be created")
    parser.add_argument("--tasks-path",
                        help="Path to file in which to write payload for all scheduled tasks")
    return parser


def run(venv, **kwargs):
    queue = taskcluster.Queue({'rootUrl': os.environ['TASKCLUSTER_PROXY_URL']})
    event = get_event(queue, event_path=kwargs["event_path"])

    task_id_map = decide(event)

    try:
        if not kwargs["dry_run"]:
            create_tasks(queue, task_id_map)
        else:
            print(json.dumps(task_id_map, indent=2))
    finally:
        if kwargs["tasks_path"]:
            with open(kwargs["tasks_path"], "w") as f:
                json.dump(task_id_map, f, indent=2)

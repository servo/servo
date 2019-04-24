import argparse
import json
import os

import jsone
import yaml

here = os.path.dirname(__file__)
root = os.path.abspath(os.path.join(here, "..", ".."))


def create_parser():
    return argparse.ArgumentParser()


def run(venv, **kwargs):
    with open(os.path.join(root, ".taskcluster.yml")) as f:
        template = yaml.safe_load(f)

    events = [("pr_event.json", "github-pull-request", "Pull Request"),
              ("master_push_event.json", "github-push", "Push to master")]

    for filename, tasks_for, title in events:
        with open(os.path.join(here, "testdata", filename)) as f:
            event = json.load(f)

        context = {"tasks_for": tasks_for,
                   "event": event,
                   "as_slugid": lambda x: x}

        data = jsone.render(template, context)
        heading = "Got %s tasks for %s" % (len(data["tasks"]), title)
        print(heading)
        print("=" * len(heading))
        for item in data["tasks"]:
            print(json.dumps(item, indent=2))
        print("")

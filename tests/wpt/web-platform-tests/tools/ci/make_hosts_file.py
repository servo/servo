import argparse
import os

from ..localpaths import repo_root

from ..serve.serve import build_config, make_hosts_file


def create_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("address", default="127.0.0.1", nargs="?",
                        help="Address that hosts should point at")
    return parser


def run(**kwargs):
    config_builder = build_config(os.path.join(repo_root, "config.json"),
                                  ssl={"type": "none"})

    with config_builder as config:
        print(make_hosts_file(config, kwargs["address"]))

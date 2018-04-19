import argparse
import os

from ..localpaths import repo_root

from ..serve.serve import load_config, make_hosts_file

def create_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("address", default="127.0.0.1", nargs="?", help="Address that hosts should point at")
    return parser

def run(**kwargs):
    config = load_config(os.path.join(repo_root, "config.default.json"),
                         os.path.join(repo_root, "config.json"))

    print(make_hosts_file(config, kwargs["address"]))

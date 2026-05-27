#!/usr/bin/env python3

import argparse
import subprocess
import requests
from urllib.request import urlretrieve
import shutil
import os

parser = argparse.ArgumentParser(description="Manage WebIDL.py checkout.")
parser.add_argument("command", choices=["update", "verify"], help="The command to run.")
args = parser.parse_args()

# set current dir to the script's directory
os.chdir(os.path.dirname(os.path.abspath(__file__)))

def get_latest_commit():
    response = requests.get("https://hg.mozilla.org/mozilla-unified/json-log")
    response.raise_for_status()
    data = response.json()
    return data["node"]

def get_current_commit():
    with open("COMMIT", "r") as f:
        return f.read().strip()

def download_and_extract(commit: str, target = "parser"):
    urlretrieve(f"https://hg-edge.mozilla.org/mozilla-unified/archive/{commit}.zip/dom/bindings/parser/", "parser.zip")
    import zipfile
    with zipfile.ZipFile("parser.zip", 'r') as zip_ref:
        zip_ref.extractall()
    shutil.rmtree(target, ignore_errors=True)
    shutil.copytree(f"mozilla-unified-{commit}/dom/bindings/parser", target)
    shutil.rmtree(f"mozilla-unified-{commit}")
    os.remove("parser.zip")

def apply_patches(dir="parser"):
    # go over the patches in order, applying them to the directory
    patches = sorted(
        f for f in os.listdir(".")
        if f.endswith(".patch")
    )
    for patch in patches:
        print(f"Applying patch: {patch}")
        subprocess.run(["git", "apply", f"{patch}", "--directory", f"components/script_bindings/third_party/WebIDL/{dir}"], check=True)

if args.command == "update":
    latest_commit = get_latest_commit()
    print(f"Latest commit: {latest_commit}")
    download_and_extract(latest_commit)
    apply_patches()
    with open("COMMIT", "w") as f:
        f.write(latest_commit)

elif args.command == "verify":
    current_commit = get_current_commit()
    print(f"Checking commit: {current_commit}")
    download_and_extract(current_commit, target="verify")
    apply_patches(dir="verify")
    import filecmp
    dif = filecmp.dircmp("parser", "verify")
    dif.report()
    shutil.rmtree("verify")
    if dif.left_only or dif.right_only or dif.diff_files:
        exit(1)

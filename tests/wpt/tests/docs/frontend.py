import argparse
import logging
import os
import subprocess
import sys

here = os.path.dirname(__file__)
wpt_root = os.path.abspath(os.path.join(here, ".."))

# Directories relative to the wpt root that we want to include in the docs
# Sphinx doesn't support including files outside of docs/ so we temporarily symlink
# these directories under docs/ whilst running the build.
link_dirs = [
    "tools/wptserve",
    "tools/certs",
    "tools/wptrunner",
    "tools/webtransport",
    "tools/third_party/pywebsocket3",
]

logger = logging.getLogger()


def link_source_dirs():
    created = set()
    failed = []
    for rel_path in link_dirs:
        rel_path = rel_path.replace("/", os.path.sep)
        src = os.path.join(wpt_root, rel_path)
        dest = os.path.join(here, rel_path)
        try:
            dest_dir = os.path.dirname(dest)
            if not os.path.exists(dest_dir):
                os.makedirs(dest_dir)
                created.add(dest_dir)
            if not os.path.exists(dest):
                os.symlink(src, dest, target_is_directory=True)
            else:
                if (not os.path.islink(dest) or
                    os.path.join(os.path.dirname(dest), os.readlink(dest)) != src):
                    # The file exists but it isn't a link or points at the wrong target
                    raise OSError("File exists")
        except Exception as e:
            failed.append((dest, e))
        else:
            created.add(dest)
    return created, failed


def unlink_source_dirs(created):
    # Sort backwards in length to remove all files before getting to directory
    for path in sorted(created, key=lambda x: -len(x)):
        # This will also remove empty parent directories
        if not os.path.islink(path) and os.path.isdir(path):
            os.removedirs(path)
        else:
            os.unlink(path)


def get_parser():
    p = argparse.ArgumentParser()
    p.add_argument("--type", default="html", help="Output type (default: html)")
    p.add_argument("--docker", action="store_true", help="Run inside the docs docker image")
    p.add_argument("--serve", default=None, nargs="?", const=8000,
                   type=int, help="Run a server on the specified port (default: 8000)")
    return p


def docker_build(tag="wpt:docs"):
    subprocess.check_call(["docker",
                           "build",
                           "--pull",
                           "--tag", tag,
                           here])

def docker_run(**kwargs):
    cmd = ["docker", "run"]
    cmd.extend(["--mount",
                 "type=bind,source=%s,target=/app/web-platform-tests" % wpt_root])
    if kwargs["serve"] is not None:
        serve = str(kwargs["serve"])
        cmd.extend(["--expose", serve, "--publish", f"{serve}:{serve}"])
    cmd.extend(["-w", "/app/web-platform-tests"])
    if os.isatty(os.isatty(sys.stdout.fileno())):
        cmd.append("-it")
    cmd.extend(["wpt:docs", "./wpt"])
    # /app/venv is created during docker build and is always active inside the
    # container.
    cmd.extend(["--venv", "/app/venv", "--skip-venv-setup"])
    cmd.extend(["build-docs", "--type", kwargs["type"]])
    if kwargs["serve"] is not None:
        cmd.extend(["--serve", str(kwargs["serve"])])
    logger.debug(" ".join(cmd))
    return subprocess.call(cmd)


def build(_venv, **kwargs):
    if kwargs["docker"]:
        docker_build()
        return docker_run(**kwargs)

    out_dir = os.path.join(here, "_build")
    try:
        created, failed = link_source_dirs()
        if failed:
            failure_msg = "\n".join(f"{dest}: {err}" for (dest, err) in failed)
            logger.error(f"Failed to create source symlinks:\n{failure_msg}")
            sys.exit(1)
        if kwargs["serve"] is not None:
            executable = "sphinx-autobuild"
            extras = ["--port", str(kwargs["serve"]),
                      "--host", "0.0.0.0",
                      "--watch", os.path.abspath(os.path.join(here, os.pardir, "resources")),
                      # Ignore changes to files specified with glob pattern
                      "--ignore", "**/flycheck_*",
                      "--ignore", "**/.*",
                      "--ignore", "**/#*",
                      "--ignore", "docs/frontend.py",
                      "--ignore", "docs/Dockerfile"]
        else:
            executable = "sphinx-build"
            extras = []
        cmd = [executable, "-n", "-v", "-b", kwargs["type"], "-j", "auto"] + extras + [here, out_dir]
        logger.debug(" ".join(cmd))
        subprocess.check_call(cmd)
    finally:
        unlink_source_dirs(created)

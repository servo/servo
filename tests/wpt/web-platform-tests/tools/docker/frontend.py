import argparse
import subprocess
import os

here = os.path.abspath(os.path.dirname(__file__))
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))

def build(*args, **kwargs):
    subprocess.check_call(["docker",
                           "build",
                           "--pull",
                           "--tag", "wpt:local",
                           here])


def parser_run():
    parser = argparse.ArgumentParser()
    parser.add_argument("--rebuild", action="store_true", help="Force rebuild of image")
    parser.add_argument("--checkout", action="store",
                        help="Revision to checkout in the image. "
                        "If this is not supplied we mount the wpt checkout on the host as "
                        "/home/test/web-platform-tests/")
    parser.add_argument("--privileged", action="store_true",
                        help="Run the image in priviledged mode (required for emulators)")
    return parser


def run(*args, **kwargs):
    if kwargs["rebuild"]:
        build()

    args = ["docker", "run"]
    args.extend(["--security-opt", "seccomp:%s" %
                 os.path.join(wpt_root, "tools", "docker", "seccomp.json")])
    if kwargs["privileged"]:
        args.append("--privileged")
    if kwargs["checkout"]:
        args.extend(["--env", "REF==%s" % kwargs["checkout"]])
    else:
        args.extend(["--mount",
                     "type=bind,source=%s,target=/home/test/web-platform-tests" % wpt_root])
    args.extend(["-it", "wpt:local"])

    proc = subprocess.Popen(args)
    proc.wait()
    return proc.returncode

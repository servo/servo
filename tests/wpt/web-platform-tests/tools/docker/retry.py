#! /usr/bin/env python
import argparse
import subprocess
import time
import sys


def get_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("--delay", action="store", type=float, default=3, help="Initial delay before retry, in seconds")
    parser.add_argument("--count", action="store", type=int, default=5, help="Total number of tries")
    parser.add_argument("--factor", action="store", type=float, default=2, help="Exponential backoff factor")
    parser.add_argument("cmd", nargs=argparse.REMAINDER)
    return parser


def iter_range(n):
    i = 0
    while i < n:
        yield i
        i += 1


def main():
    args = get_args().parse_args()

    if not args.cmd:
        print("No command supplied")
        sys.exit(1)

    retcode = None

    for n in iter_range(args.count):
        try:
            print("Running %s [try %d/%d]" % (" ".join(args.cmd), (n+1), args.count))
            subprocess.check_call(args.cmd)
        except subprocess.CalledProcessError as e:
            retcode = e.returncode
        else:
            print("Command succeeded")
            retcode = 0
            break

        if args.factor == 0:
            wait_time = (n+1) * args.delay
        else:
            wait_time = args.factor**n * args.delay
        if n < args.count - 1:
            print("Command failed, waiting %s seconds to retry" % wait_time)
            time.sleep(wait_time)
        else:
            print("Command failed, out of retries")

    sys.exit(retcode)


if __name__ == "__main__":
    main()

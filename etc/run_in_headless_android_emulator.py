#!/usr/bin/env python

# Copyright 2018 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import contextlib
import json
import os
import signal
import subprocess
import sys
import time


def main(avd_name, apk_path, *args):
    emulator_port = "5580"
    emulator_args = [
        tool_path("emulator", "emulator"),
        "@" + avd_name,
        "-wipe-data",
        "-no-window",
        "-no-snapshot",
        "-no-snapstorage",
        "-gpu", "swiftshader_indirect",
        "-port", emulator_port,
    ]
    with terminate_on_exit(emulator_args, stdout=sys.stderr) as emulator_process:
        # This is hopefully enough time for the emulator to exit
        # if it cannot start because of a configuration problem,
        # and probably more time than it needs to boot anyway
        time.sleep(2)

        if emulator_process.poll() is not None:
            # The emulator process has terminated already,
            # wait-for-device would block indefinitely
            print("Emulator did not start")
            return 1

        adb = [tool_path("platform-tools", "adb"), "-s", "emulator-" + emulator_port]

        with terminate_on_exit(adb + ["wait-for-device"]) as wait_for_device:
            wait_for_device.wait()

        # Now `adb shell` will work, but `adb install` needs a system service
        # that might still be in the midle of starting and not be responsive yet.
        wait_for_boot(adb)

        # These steps should happen before application start
        check_call(adb + ["install", "-r", apk_path])
        args = list(args)
        write_user_stylesheets(adb, args)
        write_hosts_file(adb)

        json_params = shell_quote(json.dumps(args))
        extra = "-e servoargs " + json_params
        cmd = "am start " + extra + " org.mozilla.servo/org.mozilla.servo.MainActivity"
        check_call(adb + ["shell", cmd], stdout=sys.stderr)

        # Start showing logs as soon as the application starts,
        # in case they say something useful while we wait in subsequent steps.
        logcat_args = [
            "--format=raw",  # Print no metadata, only log messages
            #"simpleservo:D",  # Show (debug level) Rust stdio
            #"*:S",  # Hide everything else
        ]
        with terminate_on_exit(adb + ["logcat"] + logcat_args) as logcat:

            # This step needs to happen after application start
            forward_webdriver(adb, args)

            # logcat normally won't exit on its own, wait until we get a SIGTERM signal.
            logcat.wait()


def tool_path(directory, bin_name):
    if "ANDROID_SDK" in os.environ:
        path = os.path.join(os.environ["ANDROID_SDK"], directory, bin_name)
        if os.path.exists(path):
            return path

    path = os.path.join(os.path.dirname(__file__), "..", "android-toolchains", "sdk",
                        directory, bin_name)
    if os.path.exists(path):
        return path

    return bin_name


@contextlib.contextmanager
def terminate_on_exit(*args, **kwargs):
    process = subprocess.Popen(*args, **kwargs)
    try:
        yield process
    finally:
        if process.poll() is None:
            # The process seems to be still running
            process.terminate()


# https://stackoverflow.com/a/38896494/1162888
def wait_for_boot(adb):
    while 1:
        with terminate_on_exit(
            adb + ["shell", "getprop", "sys.boot_completed"],
            stdout=subprocess.PIPE,
        ) as getprop:
            stdout, stderr = getprop.communicate()
            if "1" in stdout:
                return
        time.sleep(1)


def call(*args, **kwargs):
    with terminate_on_exit(*args, **kwargs) as process:
        return process.wait()


def check_call(*args, **kwargs):
    exit_code = call(*args, **kwargs)
    if exit_code != 0:
        sys.exit(exit_code)


def write_user_stylesheets(adb, args):
    data_dir = "/sdcard/Android/data/org.mozilla.servo/files"
    check_call(adb + ["shell", "mkdir -p %s" % data_dir])
    for i, (pos, path) in enumerate(extract_args("--user-stylesheet", args)):
        remote_path = "%s/user%s.css" % (data_dir, i)
        args[pos] = remote_path
        check_call(adb + ["push", path, remote_path], stdout=sys.stderr)


def write_hosts_file(adb):
    hosts_file = os.environ.get("HOST_FILE")
    if hosts_file:
        data_dir = "/sdcard/Android/data/org.mozilla.servo/files"
        check_call(adb + ["shell", "mkdir -p %s" % data_dir])
        remote_path = data_dir + "/android_hosts"
        check_call(adb + ["push", hosts_file, remote_path], stdout=sys.stderr)


def forward_webdriver(adb, args):
    webdriver_port = extract_arg("--webdriver", args)
    if webdriver_port is not None:
        # `adb forward` will start accepting TCP connections even if the other side does not.
        # (If the remote side refuses the connection,
        # adb will close the local side after accepting it.)
        # This is incompatible with wptrunner which relies on TCP connection acceptance
        # to figure out when it can start sending WebDriver requests.
        #
        # So wait until the remote side starts listening before setting up the forwarding.
        wait_for_tcp_server(adb, webdriver_port)

        port = "tcp:%s" % webdriver_port
        check_call(adb + ["forward", port, port])
        sys.stderr.write("Forwarding WebDriver port %s to the emulator\n" % webdriver_port)

    split = os.environ.get("EMULATOR_REVERSE_FORWARD_PORTS", "").split(",")
    ports = [int(part) for part in split if part]
    for port in ports:
        port = "tcp:%s" % port
        check_call(adb + ["reverse", port, port])
    if ports:
        sys.stderr.write("Reverse-forwarding ports %s\n" % ", ".join(map(str, ports)))


def extract_arg(name, args):
    for _, arg in extract_args(name, args):
        return arg


def extract_args(name, args):
    assert "=" not in name
    previous_arg_matches = False
    for i, arg in enumerate(args):
        if previous_arg_matches:
            yield i, arg
        previous_arg_matches = arg == name

        arg, sep, value = arg.partition("=")
        if arg == name and sep == "=":
            yield i, value


def wait_for_tcp_server(adb, port):
    while call(adb + ["shell", "nc -z 127.0.0.1 %s" % port], stdout=sys.stderr) != 0:
        time.sleep(1)


# Copied from Python 3.3+'s shlex.quote()
def shell_quote(arg):
    # use single quotes, and put single quotes into double quotes
    # the string $'b is then quoted as '$'"'"'b'
    return "'" + arg.replace("'", "'\"'\"'") + "'"


def interrupt(_signum, _frame):
    raise KeyboardInterrupt


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: %s avd_name apk_path [servo args...]" % sys.argv[0])
        print("Example: %s servo-x86 target/i686-linux-android/release/servo.apk https://servo.org"
              % sys.argv[0])
        sys.exit(1)

    try:
        # When `./mach test-android-startup` runs `Popen.terminate()` on this process,
        # raise an exception in order to make `finally:` blocks run
        # and also terminate sub-subprocesses.
        signal.signal(signal.SIGTERM, interrupt)
        sys.exit(main(*sys.argv[1:]))
    except KeyboardInterrupt:
        sys.exit(1)

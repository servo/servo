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
        "-gpu", "guest",
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

        check_call(adb + ["install", "-r", apk_path])
        args = list(args)
        write_user_stylesheets(adb, args)
        write_args(adb, args)
        check_call(adb + ["shell", "am start com.mozilla.servo/com.mozilla.servo.MainActivity"],
                   stdout=sys.stderr)

        logcat_args = ["RustAndroidGlueStdouterr:D", "*:S", "-v", "raw"]
        with terminate_on_exit(adb + ["logcat"] + logcat_args) as logcat:
            forward_webdriver(adb, args)
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


def write_args(adb, args):
    data_dir = "/sdcard/Android/data/com.mozilla.servo/files"
    params_file = data_dir + "/android_params"

    check_call(adb + ["shell", "mkdir -p %s" % data_dir])
    check_call(adb + ["shell", "echo 'servo' > %s" % params_file])
    for arg in args:
        check_call(adb + ["shell", "echo %s >> %s" % (shell_quote(arg), params_file)])


def write_user_stylesheets(adb, args):
    data_dir = "/sdcard/Android/data/com.mozilla.servo/files"
    check_call(adb + ["shell", "mkdir -p %s" % data_dir])
    for i, (pos, path) in enumerate(extract_args("--user-stylesheet", args)):
        remote_path = "%s/user%s.css" % (data_dir, i)
        args[pos] = remote_path
        check_call(adb + ["push", path, remote_path], stdout=sys.stderr)


def forward_webdriver(adb, args):
    webdriver_port = extract_arg("--webdriver", args)
    if webdriver_port is not None:
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
    previous_arg_matches = False
    for i, arg in enumerate(args):
        if previous_arg_matches:
            yield i, arg
        previous_arg_matches = arg == name


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

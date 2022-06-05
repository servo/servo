# mypy: allow-untyped-defs

import argparse
import os
import platform
import shutil
import subprocess

import requests
from .wpt import venv_dir

android_device = None

here = os.path.abspath(os.path.dirname(__file__))
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))


def do_delayed_imports():
    global android_device
    from mozrunner.devices import android_device
    android_device.TOOLTOOL_PATH = os.path.join(os.path.dirname(__file__),
                                                os.pardir,
                                                "third_party",
                                                "tooltool",
                                                "tooltool.py")


def get_parser_install():
    parser = argparse.ArgumentParser()
    parser.add_argument("--reinstall", action="store_true", default=False,
                        help="Force reinstall even if the emulator already exists")
    return parser


def get_parser_start():
    return get_parser_install()


def get_sdk_path(dest):
    if dest is None:
        # os.getcwd() doesn't include the venv path
        dest = os.path.join(wpt_root, venv_dir())
    dest = os.path.join(dest, 'android-sdk')
    return os.path.abspath(os.environ.get('ANDROID_SDK_PATH', dest))


def uninstall_sdk(dest=None):
    path = get_sdk_path(dest)
    if os.path.exists(path) and os.path.isdir(path):
        shutil.rmtree(path)


def install_sdk(logger, dest=None):
    sdk_path = get_sdk_path(dest)
    if os.path.isdir(sdk_path):
        logger.info("Using SDK installed at %s" % sdk_path)
        return sdk_path, False

    if not os.path.exists(sdk_path):
        os.makedirs(sdk_path)

    os_name = platform.system().lower()
    if os_name not in ["darwin", "linux", "windows"]:
        logger.critical("Unsupported platform %s" % os_name)
        raise NotImplementedError

    os_name = 'darwin' if os_name == 'macosx' else os_name
    # TODO: either always use the latest version or have some way to
    # configure a per-product version if there are strong requirements
    # to use a specific version.
    url = f'https://dl.google.com/android/repository/sdk-tools-{os_name}-4333796.zip'

    logger.info("Getting SDK from %s" % url)
    temp_path = os.path.join(sdk_path, url.rsplit("/", 1)[1])
    try:
        with open(temp_path, "wb") as f:
            with requests.get(url, stream=True) as resp:
                shutil.copyfileobj(resp.raw, f)

        # Python's zipfile module doesn't seem to work here
        subprocess.check_call(["unzip", temp_path], cwd=sdk_path)
    finally:
        os.unlink(temp_path)

    return sdk_path, True


def install_android_packages(logger, sdk_path, no_prompt=False):
    sdk_manager_path = os.path.join(sdk_path, "tools", "bin", "sdkmanager")
    if not os.path.exists(sdk_manager_path):
        raise OSError("Can't find sdkmanager at %s" % sdk_manager_path)

    packages = ["platform-tools",
                "build-tools;31.0.0",
                "platforms;android-31",
                "emulator"]

    # TODO: make this work non-internactively
    logger.info("Installing SDK packages")
    cmd = [sdk_manager_path] + packages

    proc = subprocess.Popen(cmd, stdin=subprocess.PIPE)
    if no_prompt:
        data = "Y\n" * 100 if no_prompt else None
        proc.communicate(data)
    else:
        proc.wait()
    if proc.returncode != 0:
        raise subprocess.CalledProcessError(proc.returncode, cmd)


def get_emulator(sdk_path, device_serial=None):
    if android_device is None:
        do_delayed_imports()
    if "ANDROID_SDK_ROOT" not in os.environ:
        os.environ["ANDROID_SDK_ROOT"] = sdk_path
    substs = {"top_srcdir": wpt_root, "TARGET_CPU": "x86"}
    emulator = android_device.AndroidEmulator("*", substs=substs, device_serial=device_serial)
    emulator.emulator_path = os.path.join(sdk_path, "emulator", "emulator")
    return emulator


def install(logger, reinstall=False, no_prompt=False, device_serial=None):
    if reinstall:
        uninstall_sdk()

    dest, new_install = install_sdk(logger)
    if new_install:
        install_android_packages(logger, dest, no_prompt)

    if "ANDROID_SDK_ROOT" not in os.environ:
        os.environ["ANDROID_SDK_ROOT"] = dest

    emulator = get_emulator(dest, device_serial=device_serial)
    return emulator


def start(logger, emulator=None, reinstall=False, device_serial=None):
    if reinstall:
        install(reinstall=True)

    sdk_path = get_sdk_path(None)

    if emulator is None:
        emulator = get_emulator(sdk_path, device_serial=device_serial)

    if not emulator.check_avd():
        logger.critical("Android AVD not found, please run |mach bootstrap|")
        raise NotImplementedError

    emulator.start()
    emulator.wait_for_start()
    return emulator


def run_install(venv, **kwargs):
    try:
        import logging
        logging.basicConfig()
        logger = logging.getLogger()

        install(logger, **kwargs)
    except Exception:
        import traceback
        traceback.print_exc()
        import pdb
        pdb.post_mortem()


def run_start(venv, **kwargs):
    try:
        import logging
        logging.basicConfig()
        logger = logging.getLogger()

        start(logger, **kwargs)
    except Exception:
        import traceback
        traceback.print_exc()
        import pdb
        pdb.post_mortem()

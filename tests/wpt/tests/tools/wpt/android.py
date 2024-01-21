# mypy: allow-untyped-defs

import argparse
import os
import platform
import signal
import shutil
import subprocess
import threading

import requests
from .wpt import venv_dir

android_device = None

here = os.path.abspath(os.path.dirname(__file__))
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))


NDK_VERSION = "r25c"
CMDLINE_TOOLS_VERSION_STRING = "11.0"
CMDLINE_TOOLS_VERSION = "9644228"

AVD_MANIFEST_X86_64 = {
    "emulator_package": "system-images;android-24;default;x86_64",
    "emulator_avd_name": "mozemulator-x86_64",
    "emulator_extra_args": [
        "-skip-adb-auth",
        "-verbose",
        "-show-kernel",
        "-ranchu",
        "-selinux", "permissive",
        "-memory", "3072",
        "-cores", "4",
        "-skin", "800x1280",
        "-gpu", "on",
        "-no-snapstorage",
        "-no-snapshot",
        "-no-window",
        "-no-accel",
        "-prop", "ro.test_harness=true"
    ],
    "emulator_extra_config": {
        "hw.keyboard": "yes",
        "hw.lcd.density": "320",
        "disk.dataPartition.size": "4000MB",
        "sdcard.size": "600M"
    }
}


def do_delayed_imports(paths):
    global android_device
    from mozrunner.devices import android_device

    android_device.TOOLTOOL_PATH = os.path.join(os.path.dirname(__file__),
                                                os.pardir,
                                                "third_party",
                                                "tooltool",
                                                "tooltool.py")
    android_device.EMULATOR_HOME_DIR = paths["emulator_home"]


def get_parser_install():
    parser = argparse.ArgumentParser()
    parser.add_argument("--path", dest="dest", action="store", default=None,
                        help="Root path to use for emulator tooling")
    parser.add_argument("--reinstall", action="store_true", default=False,
                        help="Force reinstall even if the emulator already exists")
    parser.add_argument("--prompt", action="store_true",
                        help="Enable confirmation prompts")
    parser.add_argument("--no-prompt", dest="prompt", action="store_false",
                        help="Skip confirmation prompts")
    return parser


def get_parser_start():
    parser = get_parser_install()
    parser.add_argument("--device-serial", action="store", default=None,
                        help="Device serial number for Android emulator, if not emulator-5554")
    return parser


def install_fixed_emulator_version(logger, paths):
    # Downgrade to a pinned emulator version
    # See https://developer.android.com/studio/emulator_archive for what we're doing here
    from xml.etree import ElementTree

    version = "32.1.15"
    urls = {"linux": "https://redirector.gvt1.com/edgedl/android/repository/emulator-linux_x64-10696886.zip"}

    os_name = platform.system().lower()
    if os_name not in urls:
        logger.error(f"Don't know how to install old emulator for {os_name}, using latest version")
        # For now try with the latest version if this fails
        return

    logger.info(f"Downgrading emulator to {version}")
    url = urls[os_name]

    emulator_path = os.path.join(paths["sdk"], "emulator")
    latest_emulator_path = os.path.join(paths["sdk"], "emulator_latest")
    os.rename(emulator_path, latest_emulator_path)

    download_and_extract(url, paths["sdk"])
    package_path = os.path.join(emulator_path, "package.xml")
    shutil.copyfile(os.path.join(latest_emulator_path, "package.xml"),
                    package_path)

    with open(package_path) as f:
        tree = ElementTree.parse(f)
    node = tree.find("localPackage").find("revision")
    assert len(node) == 3
    parts = version.split(".")
    for version_part, node in zip(parts, node):
        node.text = version_part
    with open(package_path, "wb") as f:
        tree.write(f, encoding="utf8")


def get_paths(dest):
    os_name = platform.system().lower()

    if dest is None:
        # os.getcwd() doesn't include the venv path
        base_path = os.path.join(wpt_root, venv_dir(), "android")
    else:
        base_path = dest

    sdk_path = os.environ.get("ANDROID_SDK_HOME", os.path.join(base_path, f"android-sdk-{os_name}"))
    avd_path = os.environ.get("ANDROID_AVD_HOME", os.path.join(sdk_path, ".android", "avd"))
    return {
        "base": base_path,
        "sdk": sdk_path,
        "sdk_tools": os.path.join(sdk_path, "cmdline-tools", CMDLINE_TOOLS_VERSION_STRING),
        "avd": avd_path,
        "emulator_home": os.path.dirname(avd_path)
    }


def get_sdk_manager_path(paths):
    os_name = platform.system().lower()
    file_name = "sdkmanager"
    if os_name.startswith("win"):
        file_name += ".bat"
    return os.path.join(paths["sdk_tools"], "bin", file_name)


def get_avd_manager(paths):
    os_name = platform.system().lower()
    file_name = "avdmanager"
    if os_name.startswith("win"):
        file_name += ".bat"
    return os.path.join(paths["sdk_tools"], "bin", file_name)


def uninstall_sdk(paths):
    if os.path.exists(paths["sdk"]) and os.path.isdir(paths["sdk"]):
        shutil.rmtree(paths["sdk"])


def get_os_tag(logger):
    os_name = platform.system().lower()
    if os_name not in ["darwin", "linux", "windows"]:
        logger.critical("Unsupported platform %s" % os_name)
        raise NotImplementedError

    if os_name == "macosx":
        return "darwin"
    if os_name == "windows":
        return "win"
    return "linux"


def download_and_extract(url, path):
    if not os.path.exists(path):
        os.makedirs(path)
    temp_path = os.path.join(path, url.rsplit("/", 1)[1])
    try:
        with open(temp_path, "wb") as f:
            with requests.get(url, stream=True) as resp:
                shutil.copyfileobj(resp.raw, f)

        # Python's zipfile module doesn't seem to work here
        subprocess.check_call(["unzip", temp_path], cwd=path)
    finally:
        if os.path.exists(temp_path):
            os.unlink(temp_path)


def install_sdk(logger, paths):
    if os.path.isdir(paths["sdk_tools"]):
        logger.info("Using SDK installed at %s" % paths["sdk_tools"])
        return False

    if not os.path.exists(paths["sdk"]):
        os.makedirs(paths["sdk"])

    download_path = os.path.dirname(paths["sdk_tools"])

    url = f'https://dl.google.com/android/repository/commandlinetools-{get_os_tag(logger)}-{CMDLINE_TOOLS_VERSION}_latest.zip'
    logger.info("Getting SDK from %s" % url)

    download_and_extract(url, download_path)
    os.rename(os.path.join(download_path, "cmdline-tools"), paths["sdk_tools"])

    return True


def install_android_packages(logger, paths, packages, prompt=True):
    sdk_manager = get_sdk_manager_path(paths)
    if not os.path.exists(sdk_manager):
        raise OSError(f"Can't find sdkmanager at {sdk_manager}")

    # TODO: make this work non-internactively
    logger.info(f"Installing Android packages {' '.join(packages)}")
    cmd = [sdk_manager] + packages

    input_data = None if prompt else "\n".join(["y"] * 100).encode("UTF-8")
    subprocess.run(cmd, check=True, input=input_data)


def install_avd(logger, paths, prompt=True):
    avd_manager = get_avd_manager(paths)
    avd_manifest = AVD_MANIFEST_X86_64

    install_android_packages(logger, paths, [avd_manifest["emulator_package"]], prompt=prompt)

    cmd = [avd_manager,
           "--verbose",
           "create",
           "avd",
           "--force",
           "--name",
           avd_manifest["emulator_avd_name"],
           "--package",
           avd_manifest["emulator_package"]]
    input_data = None if prompt else b"no"
    subprocess.run(cmd, check=True, input=input_data)


def get_emulator(paths, device_serial=None):
    if android_device is None:
        do_delayed_imports(paths)

    substs = {"top_srcdir": wpt_root, "TARGET_CPU": "x86"}
    emulator = android_device.AndroidEmulator(substs=substs,
                                              device_serial=device_serial,
                                              verbose=True)
    emulator.emulator_path = os.path.join(paths["sdk"], "emulator", "emulator")
    return emulator


class Environ:
    def __init__(self, **kwargs):
        self.environ = None
        self.set_environ = kwargs

    def __enter__(self):
        self.environ = os.environ.copy()
        for key, value in self.set_environ.items():
            if value is None:
                if key in os.environ:
                    del os.environ[key]
            else:
                os.environ[key] = value

    def __exit__(self, *args, **kwargs):
        os.environ = self.environ


def android_environment(paths):
    return Environ(ANDROID_EMULATOR_HOME=paths["emulator_home"],
                   ANDROID_AVD_HOME=paths["avd"],
                   ANDROID_SDK_ROOT=paths["sdk"],
                   ANDROID_SDK_HOME=paths["sdk"])


def install(logger, dest=None, reinstall=False, prompt=True):
    paths = get_paths(dest)

    with android_environment(paths):

        if reinstall:
            uninstall_sdk(paths)

        new_install = install_sdk(logger, paths)

        if new_install:
            packages = ["platform-tools",
                        "build-tools;34.0.0",
                        "platforms;android-34",
                        "emulator"]

            install_android_packages(logger, paths, packages, prompt=prompt)

            install_avd(logger, paths, prompt=prompt)

            install_fixed_emulator_version(logger, paths)

        emulator = get_emulator(paths)
    return emulator


def cancel_start(thread_id):
    def cancel_func():
        raise signal.pthread_kill(thread_id, signal.SIGINT)
    return cancel_func


def start(logger, dest=None, reinstall=False, prompt=True, device_serial=None):
    paths = get_paths(dest)

    with android_environment(paths):
        install(logger, dest=dest, reinstall=reinstall, prompt=prompt)

        emulator = get_emulator(paths, device_serial=device_serial)

        if not emulator.check_avd():
            logger.critical("Android AVD not found, please run |wpt install-android-emulator|")
            raise OSError

        emulator.start()
        timer = threading.Timer(300, cancel_start(threading.get_ident()))
        timer.start()
        emulator.wait_for_start()
        timer.cancel()
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

#!/usr/bin/env python3

# Copyright 2026 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import argparse
import threading
import time
import csv
from dataclasses import dataclass, field
import subprocess
from typing import List, Optional
import matplotlib.pyplot as plt
import datetime as dt
from pathlib import Path
from selenium import webdriver
import sys
from hdc_py.hdc import HarmonyDeviceConnector
from common_function_for_servo_test import create_driver

PACKAGE_NAME = "org.servo.servo"


### Use this MemoryLoggingOptions dataclass definition to setup default values
@dataclass
class MemoryLoggingOptions:
    verbose: bool = False
    frequency: int = 2
    pid: int = None
    log_to_file: bool = False
    plot: bool = False
    file_name: str = "memory_usage_plotter"
    pre_time: int = 0
    post_time: int = 0
    set_minimal_history: bool = False
    from_dump: str = None
    mode: str = "collect"
    reset_tab: str = None
    create_own_webdriver: bool = False


@dataclass
class MemoryInfo:
    vm_rss_kb: int
    vm_hwm_kb: int
    vm_size_kb: int
    vm_swap_kb: int
    rss_anon_kb: int
    rss_file_kb: int
    rss_shmem_kb: int

    @property
    def vm_rss_mb(self) -> float:
        return self.vm_rss_kb / 1024.0

    @property
    def total_working_set_kb(self) -> int:
        return self.vm_rss_kb + self.vm_swap_kb

    def __str__(self) -> str:
        return (
            f"Total Memory: {self.total_working_set_kb / 1024:.2f} MB "
            f"(RSS={self.vm_rss_kb / 1024:.2f} MB, "
            f"Swap={self.vm_swap_kb / 1024:.2f} MB)"
        )

    def get_rss_mb(self) -> float:
        return self.vm_rss_kb / 1024

    __repr__ = __str__


def get_memory_info(pid: int) -> Optional[MemoryInfo]:
    cmd = ["hdc", "shell", "cat", f"/proc/{pid}/status"]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=True,
        )
    except subprocess.CalledProcessError:
        return None

    fields = {
        "VmRSS": 0,
        "VmHWM": 0,
        "VmSize": 0,
        "VmSwap": 0,
        "RssAnon": 0,
        "RssFile": 0,
        "RssShmem": 0,
    }

    for line in result.stdout.splitlines():
        for key in fields:
            if line.startswith(key + ":"):
                parts = line.split()
                if len(parts) >= 2:
                    fields[key] = int(parts[1])

    return MemoryInfo(
        vm_rss_kb=fields["VmRSS"],
        vm_hwm_kb=fields["VmHWM"],
        vm_size_kb=fields["VmSize"],
        vm_swap_kb=fields["VmSwap"],
        rss_anon_kb=fields["RssAnon"],
        rss_file_kb=fields["RssFile"],
        rss_shmem_kb=fields["RssShmem"],
    )


class InvalidInputFile(Exception):
    pass


def raise_if_input_invalid(file_path: str) -> None:
    p = Path(file_path)

    if not p.is_file():
        raise InvalidInputFile(f"{file_path} is not a file")
    with p.open("r", encoding="utf-8") as f:
        first_line = f.readline().strip()

    if not first_line.startswith("timestamp,rss"):
        raise InvalidInputFile("Invalid CSV header")


def pidof(package_name: str, hdc: HarmonyDeviceConnector) -> List[int]:
    completed_process = hdc.cmd("pidof " + package_name, capture_output=True, encoding="utf-8")
    output = str(completed_process.stdout)
    if not output:
        return []

    return [int(pid) for pid in output.split() if pid.isdigit()]


@dataclass(slots=True)
class MemorySample:
    timestamp: float  # epoch seconds (time.time())
    RSS_MB: float
    Swap_MB: float
    event_name: Optional[str]


@dataclass(slots=True)
class MemoryLog:
    samples: List[MemorySample] = field(default_factory=list)

    def add(self, sample: MemorySample):
        self.samples.append(sample)

    def clear(self):
        self.samples.clear()


def load_memory_log(path: str) -> MemoryLog:
    log = MemoryLog()

    with open(path, newline="") as f:
        reader = csv.DictReader(f)

        for row in reader:
            event: Optional[str] = row["event"] or None

            sample = MemorySample(
                timestamp=float(row["timestamp"]),
                RSS_MB=float(row["rss (kb)"]),
                Swap_MB=float(row["swap (kb)"]),
                event_name=event,
            )

            log.add(sample)

    return log


class WebDriverIsNotSet(Exception):
    pass


class NonBlockingMemoryLogging:
    def set_webdriver(self, webdriver: webdriver):
        self.driver = webdriver

    def from_dump(self):
        self.log = load_memory_log(self.options.from_dump)
        self.plot_memory_log()

    def __init__(self, options: MemoryLoggingOptions = None):
        # Defaults:
        self.options = MemoryLoggingOptions()
        if options is not None:
            self.options = options
        self.driver = None
        if self.options.create_own_webdriver:
            self.driver = create_driver(timeout=1)
            if self.driver is None:
                print("Doublecheck that servo is running and has `--psn=--webdriver`")
                sys.exit(0)

        if self.options.verbose:
            print(f"Memory plotter options: {self.options}")

        # check for the `file` mode
        if options.mode is None:
            print("No mode has been specified. Exiting")
            sys.exit(1)
        if options.mode == "plot" and options.from_dump is not None:
            raise_if_input_invalid(options.from_dump)
            self.from_dump()
            sys.exit(0)
        self.log = MemoryLog()
        if self.options.log_to_file:
            self.csv_file = open(self.options.file_name + ".csv", "w", newline="", encoding="utf-8")
            self.writer = csv.writer(self.csv_file)
            self.writer.writerow(["timestamp", "rss (kb)", "swap (kb)", "event"])
        self._stop_event = threading.Event()
        self._thread = threading.Thread(
            target=self._run,
            daemon=True,
        )

    def __enter__(self):
        self.start()
        self.csv_file = None
        return self

    def __exit__(self, exc_type, exc, tb):
        self.stop()
        if self.csv_file:
            self.csv_file.close()
        if self.driver is not None:
            self.driver.quit()
        return False

    def start(self):
        if self.options.pid is None:
            try:
                self.hdc = HarmonyDeviceConnector()
                self.options.pid = get_servo_pid(PACKAGE_NAME, self.hdc)
            except (MoreThanOneInstanceOfServo, ProcessLookupError) as e:
                print(f"Failed to get servo PID: {e}")
            else:
                self._thread.start()
                if self.options.pre_time is not None:
                    self.verbose_print(f"started sampling, with {self.options.pre_time}s delay")
                    time.sleep(abs(self.options.pre_time))
                self.event("start")

    def stop(self):
        self.event("stop")
        if self.options.post_time is not None:
            self.verbose_print(f"post-logging for {self.options.post_time}s...")
            if self.options.reset_tab:
                if self.driver is None:
                    raise WebDriverIsNotSet("The `-r` argument or Tab Reset was passed, but the webdriver is not set")
                self.verbose_print(f"Setting the url to reset_tab: {self.options.reset_tab}")
                if self.options.reset_tab is not str:
                    self.options.reset_tab = "about:blank"
                self.driver.get(self.options.reset_tab)
            time.sleep(self.options.post_time)
        self.verbose_print("Memory plotter stop")
        if self.options.pid is not None:
            self._stop_event.set()
            self._thread.join()
            if self.options.verbose:
                print(self.log)
            if self.options.log_to_file:
                self.csv_file.close()
            if self.options.plot:
                self.plot_memory_log()

    def _run(self):
        while not self._stop_event.is_set():
            self.event()
            time.sleep(1 / self.options.frequency)

    def event(self, event_name: str = None):
        memory_point = get_memory_info(self.options.pid)
        if self.options.verbose:
            print(memory_point)
        sample = MemorySample(
            timestamp=time.time(),
            RSS_MB=memory_point.vm_rss_kb / 1024,
            Swap_MB=memory_point.vm_swap_kb / 1024,
            event_name=event_name,
        )
        if self.options.log_to_file:
            self.writer.writerow([sample.timestamp, sample.RSS_MB, sample.Swap_MB, event_name])
            self.csv_file.flush()
        self.log.add(sample)

    def verbose_print(self, to_print: str) -> None:
        if self.options.verbose:
            print(to_print)

    def plot_memory_log(self) -> None:
        if not self.log.samples:
            raise ValueError("MemoryLog is empty")

        # Extract data
        times = [dt.datetime.fromtimestamp(s.timestamp) for s in self.log.samples]
        rss = [s.RSS_MB for s in self.log.samples]
        swap = [s.Swap_MB for s in self.log.samples]
        total = [r + s for r, s in zip(rss, swap)]

        # Plot
        plt.figure(figsize=(10, 5))
        plt.plot(times, rss, label="RSS (MB)", linewidth=2)
        plt.plot(times, swap, label="Swap (MB)", linewidth=2)
        plt.plot(
            times,
            total,
            label="Total (MB)",
            linewidth=2,
            linestyle="--",
        )

        for t, tot, sample in zip(times, total, self.log.samples):
            pos = (0, 8)
            color = "red"
            if sample.event_name:
                if sample.event_name in ["start", "stop"]:
                    pos = (0, -12)
                    color = "blue"
                plt.scatter(t, tot, color=color, zorder=5)
                plt.annotate(
                    sample.event_name,
                    (t, tot),
                    textcoords="offset points",
                    xytext=pos,
                    ha="center",
                    fontsize=9,
                    color=color,
                )

        date_str = dt.datetime.now().strftime("%d %b %Y")
        plt.xlabel(f"Time ({date_str})")
        plt.ylabel("Memory (MB)")
        if self.options.file_name is not None:
            plt.title(self.options.file_name)
        else:
            plt.title("Memory Usage Over Time")
        plt.ylim(bottom=0)
        plt.legend()
        plt.grid(True)
        plt.tight_layout()
        plt.savefig(self.options.file_name, dpi=150)


class MoreThanOneInstanceOfServo(Exception):
    pass


def get_servo_pid(package_name: str, hdc: HarmonyDeviceConnector) -> int | None:
    pids = pidof(package_name, hdc)
    if not pids:
        raise ProcessLookupError(f"No running instances of {package_name}")
    if len(pids) > 1:
        raise MoreThanOneInstanceOfServo(f"Expected only 1 instance of {package_name}, found {len(pids)}")
    return pids[0]


if __name__ == "__main__":
    default_options = MemoryLoggingOptions()
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="print each time sample is taken",
    )

    subparsers = parser.add_subparsers(dest="mode", required=True)

    collect = subparsers.add_parser("collect", help="collect data from phone")
    collect.add_argument(
        "-l",
        "--log-to-file",
        action="store_true",
        help="store log data in csv",
    )
    collect.add_argument(
        "-p",
        "--plot",
        action="store_true",
        help="create plot after collection",
    )
    collect.add_argument(
        "-r",
        "--reset-tab",
        nargs="?",
        const="about:blank",
        default=None,
        help="on stop event, reset the tab to `about:blank` or to specified str value",
    )
    collect.add_argument(
        "-m",
        "--set-reset-to-memory",
        action="store_true",
        help="when doing reset-tab, reset the tab to `about:memory` instead of `about:blank`",
    )

    collect.add_argument(
        "--pre-time",
        type=int,
        default=default_options.pre_time,
        help="time in positive seconds of sampling before starting the test",
    )
    collect.add_argument(
        "--post-time",
        type=int,
        default=default_options.post_time,
    )
    collect.add_argument(
        "--file-name",
        type=str,
        default=default_options.file_name,
    )
    collect.add_argument(
        "--frequency",
        type=int,
        default=default_options.frequency,
    )
    collect.add_argument("--pid", type=int)

    plot = subparsers.add_parser("plot", help="plot from csv dump")
    plot.add_argument(
        "from_dump",
        metavar="from-dump",
        type=str,
        help="csv file to analyze",
    )
    plot.add_argument(
        "--file-name",
        type=str,
        default=default_options.file_name,
    )

    args = parser.parse_args()
    args.create_own_webdriver = True
    with NonBlockingMemoryLogging(args) as worker:
        time.sleep(2)
        print("Exiting memory_usage_plotter.py")

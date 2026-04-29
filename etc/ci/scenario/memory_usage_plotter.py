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
from typing import List, Optional, Self, Type
from dataclasses import dataclass, field, fields
import subprocess
import matplotlib.pyplot as plt
import matplotlib.dates as mdates
import datetime as dt
from pathlib import Path
from selenium import webdriver
from hdc_py.hdc import HarmonyDeviceConnector  # type: ignore[import-untyped]
from enum import Enum
from types import TracebackType

PACKAGE_NAME = "org.servo.servo"
SERVO_PROCESS_NAME = "servoshell"


class HostOptions(str, Enum):
    LINUX = "linux"
    OHOS = "ohos"
    MACOS = "macos"


### Use this MemoryLoggingOptions dataclass definition to setup default values
@dataclass
class MemoryLoggingOptions:
    verbose: bool = False
    frequency: float = 2
    pid: Optional[int] = None
    log_to_file: bool = False
    plot: bool = False
    file_name: str = "memory_usage_plotter"
    test_name: Optional[str] = None
    pre_time: float = 0
    post_time: float = 0
    set_minimal_history: bool = False
    from_dump: Optional[str] = None
    mode: str = "collect"
    reset_tab: Optional[str | bool] = None
    create_own_webdriver: bool = False
    host: HostOptions | str = HostOptions.LINUX
    url: Optional[str] = None


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


def get_memory_info(pid: int, host: HostOptions | str) -> Optional[MemoryInfo]:
    result = None
    cmd = ["echo error"]
    if host == HostOptions.OHOS:
        cmd = ["hdc", "shell", "cat", f"/proc/{pid}/status"]
    if host == HostOptions.MACOS:
        # Beware that the macos does not expose the swap usage per PID be default
        cmd = ["ps", "-p", str(pid), "-o", "rss="]
    if host == HostOptions.LINUX:
        cmd = ["cat", f"/proc/{pid}/status"]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=True,
        )
    except subprocess.CalledProcessError as e:
        raise RuntimeError(f"Error running '{' '.join(cmd)}': {e}") from e

    fields = {
        "VmRSS": 0,
        "VmHWM": 0,
        "VmSize": 0,
        "VmSwap": 0,
        "RssAnon": 0,
        "RssFile": 0,
        "RssShmem": 0,
    }

    if host == HostOptions.MACOS:
        parts = result.stdout.split()
        if len(parts) >= 1:
            fields["VmRSS"] = int(parts[0])
            fields["VmSwap"] = 0
        else:
            return None

    if host == HostOptions.OHOS or host == HostOptions.LINUX:
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


def pidof(process_name: str, hdc: Optional[HarmonyDeviceConnector] = None) -> List[int]:
    if hdc is not None:
        completed_process = hdc.cmd("pidof " + process_name, capture_output=True, encoding="utf-8")
        output = str(completed_process.stdout)
        if not output:
            return []

        return [int(pid) for pid in output.split() if pid.isdigit()]
    else:
        result = subprocess.run(
            ["pgrep", process_name],
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            return []
        return [int(pid) for pid in result.stdout.strip().split()]


@dataclass(slots=True)
class MemorySample:
    timestamp: float  # epoch seconds (time.time())
    RSS_MB: float
    Swap_MB: float
    event_name: Optional[str]


@dataclass(slots=True)
class MemoryLog:
    samples: List[MemorySample] = field(default_factory=list)

    def add(self, sample: MemorySample) -> None:
        self.samples.append(sample)

    def clear(self) -> None:
        self.samples.clear()


def load_memory_log(path: str) -> MemoryLog:
    log = MemoryLog()

    with open(path, newline="") as f:
        reader = csv.DictReader(f)

        for row in reader:
            event: Optional[str] = row["event"] or None

            # Accept both the old kb headers and the current mb headers.
            rss = row.get("rss (mb)") or row.get("rss (kb)")
            swap = row.get("swap (mb)") or row.get("swap (kb)")
            if rss is None or swap is None:
                raise InvalidInputFile("Invalid CSV columns")

            sample = MemorySample(
                timestamp=float(row["timestamp"]),
                RSS_MB=float(rss),
                Swap_MB=float(swap),
                event_name=event,
            )

            log.add(sample)

    return log


class WebDriverIsNotSet(Exception):
    pass


class NonBlockingMemoryLogging:
    def set_webdriver(self, driver: webdriver.Remote) -> None:
        self.driver: Optional[webdriver.Remote] = driver

    def from_dump(self) -> None:
        if self.options.from_dump is not None:
            self.log = load_memory_log(self.options.from_dump)
            self.plot_memory_log()
        else:
            print("Dump file was not set")

    def __init__(
        self,
        options: Optional[MemoryLoggingOptions] = None,
        host: Optional[HostOptions] = None,
        driver: Optional[webdriver.Remote] = None,
    ) -> None:
        # Defaults:
        self._loaded_from_dump = False
        self.driver = None
        self.hdc = None
        self.csv_file = None
        self.options = MemoryLoggingOptions()
        if options is not None:
            self.options = options
            if options.from_dump is not None:
                raise_if_input_invalid(options.from_dump)
                self.from_dump()
                self._loaded_from_dump = True
        if host is not None:
            self.options.host = host
        if driver is not None:
            self.driver = driver
        if self.options.verbose:
            print(f"Memory plotter options: {self.options}")

        self.log = MemoryLog()
        if self.options.log_to_file:
            self.csv_file = open(self.options.file_name + ".csv", "w", newline="", encoding="utf-8")
            self.writer = csv.writer(self.csv_file)
            self.writer.writerow(["timestamp", "rss (mb)", "swap (mb)", "event"])
        self._stop_event = threading.Event()
        self._thread = threading.Thread(
            target=self._run,
            daemon=True,
        )

    def __enter__(self) -> Self:
        if not self._loaded_from_dump:
            self.start()
        return self

    def __exit__(
        self,
        exc_type: Type[BaseException] | None,
        exc: BaseException | None,
        tb: TracebackType | None,
    ) -> None:
        if not self._loaded_from_dump:
            self.stop()
        if self.csv_file:
            self.csv_file.close()
        if self.driver is not None:
            self.driver.quit()

    def start(self) -> None:
        if self.options.pid is None:
            try:
                if self.options.host == HostOptions.OHOS:
                    self.hdc = HarmonyDeviceConnector()
                    self.options.pid = get_servo_pid(PACKAGE_NAME, self.hdc)
                else:
                    self.options.pid = get_servo_pid(SERVO_PROCESS_NAME)
            except (MoreThanOneInstanceOfServo, ProcessLookupError) as e:
                raise RuntimeError(f"Failed to get servo PID: {e}") from e
        self._thread.start()
        if self.options.pre_time is not None:
            self.verbose_print(f"started sampling, with {self.options.pre_time}s delay")
            time.sleep(abs(self.options.pre_time))
        self.event("start")

    def stop(self) -> None:
        self.event("stop")
        if self.options.post_time is not None:
            self.verbose_print(f"post-logging for {self.options.post_time}s...")
            if self.options.reset_tab:
                if self.driver is None:
                    raise WebDriverIsNotSet("The `-r` argument or Tab Reset was passed, but the webdriver is not set")
                self.verbose_print(f"Setting the url to reset_tab: {self.options.reset_tab}")
                if not isinstance(self.options.reset_tab, str):
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
                if self.csv_file:
                    self.csv_file.close()
            if self.options.plot:
                self.plot_memory_log()

    def _run(self) -> None:
        while not self._stop_event.is_set():
            self.event()
            time.sleep(1 / self.options.frequency)

    def event(self, event_name: Optional[str] = None) -> None:
        if self.options.pid is not None:
            memory_point = get_memory_info(self.options.pid, self.options.host)
            if memory_point is not None:
                if self.options.verbose:
                    print(f"memory_point: {memory_point}")
                sample = MemorySample(
                    timestamp=time.time(),
                    RSS_MB=memory_point.vm_rss_kb / 1024,
                    Swap_MB=memory_point.vm_swap_kb / 1024,
                    event_name=event_name,
                )
                if self.options.log_to_file:
                    self.writer.writerow([sample.timestamp, sample.RSS_MB, sample.Swap_MB, event_name])
                    if self.csv_file:
                        self.csv_file.flush()
                self.log.add(sample)

    def verbose_print(self, to_print: str) -> None:
        if self.options.verbose:
            print(to_print)

    def plot_memory_log(self) -> None:
        if not self.log.samples:
            raise ValueError("MemoryLog is empty")

        plot_path = Path(self.options.file_name)
        plot_stem = plot_path.stem if plot_path.suffix else plot_path.name
        if self.options.test_name:
            plot_stem = f"{plot_stem}_{self.options.test_name}"
        plot_output_path = plot_path.with_name(f"{plot_stem}.jpg")

        # Extract data
        times = mdates.date2num([dt.datetime.fromtimestamp(s.timestamp) for s in self.log.samples])
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
        ax = plt.gca()
        ax.xaxis.set_major_formatter(mdates.DateFormatter("%d %b %H:%M:%S"))
        plt.gcf().autofmt_xdate()
        plt.ylim(bottom=0)
        plt.legend()
        plt.grid(True)
        plt.tight_layout()
        plt.savefig(plot_output_path, dpi=150)


class MoreThanOneInstanceOfServo(Exception):
    pass


def get_servo_pid(process_name: str, hdc: Optional[HarmonyDeviceConnector] = None) -> int | None:
    pids = pidof(process_name, hdc)
    if not pids:
        raise ProcessLookupError(f"No running instances of {process_name}, or host=HostOption.XXX is not set")
    if len(pids) > 1:
        raise MoreThanOneInstanceOfServo(f"Expected only 1 instance of {process_name}, found {len(pids)}")
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

    parser.add_argument(
        "from_dump",
        metavar="dump-file",
        type=str,
        help="csv file to analyze",
    )
    parser.add_argument(
        "--file-name",
        type=str,
        default=default_options.file_name,
    )

    args = parser.parse_args()
    options_kwargs = {
        field.name: getattr(args, field.name) for field in fields(MemoryLoggingOptions) if hasattr(args, field.name)
    }
    options = MemoryLoggingOptions(**options_kwargs)
    options.create_own_webdriver = True
    with NonBlockingMemoryLogging(options) as worker:
        time.sleep(2)
        print("Exiting memory_usage_plotter.py")

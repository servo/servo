#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import argparse
import matplotlib.pyplot as plt
import numpy as np
import os
from os import path
import sys
import warnings

HB_LOG_IDX_START_TIME = 7
HB_LOG_IDX_END_TIME = HB_LOG_IDX_START_TIME + 1
HB_LOG_IDX_START_ENERGY = 14
HB_LOG_IDX_END_ENERGY = HB_LOG_IDX_START_ENERGY + 1

ENERGY_PROFILER_NAME = 'ApplicationHeartbeat'
SUMMARY_OUTPUT = "summary.txt"
SUMMARY_TIME_IDX = 8
SUMMARY_ENERGY_IDX = SUMMARY_TIME_IDX + 1
SUMMARY_POWER_IDX = SUMMARY_ENERGY_IDX + 1


def autolabel(rects, ax):
    """Attach some text labels.
    """
    for rect in rects:
        ax.text(rect.get_x() + rect.get_width() / 2., 1.05 * rect.get_height(), '', ha='center', va='bottom')


def plot_raw_totals(config, plot_data, max_time, max_time_std, max_energy, max_energy_std, output_dir, normalize):
    """Plot the raw totals for a configuration.

    Keyword arguments:
    config -- configuration name
    plot_data -- (profiler name, total_time, total_time_std, total_energy, total_energy_std)
    max_time, max_time_std, max_energy, max_energy_std -- single values
    normalize -- True/False
    """
    plot_data = sorted(plot_data)
    keys = [p for (p, tt, tts, te, tes) in plot_data]
    total_times = [tt for (p, tt, tts, te, tes) in plot_data]
    total_times_std = [tts for (p, tt, tts, te, tes) in plot_data]
    total_energies = [te for (p, tt, tts, te, tes) in plot_data]
    total_energies_std = [tes for (p, tt, tts, te, tes) in plot_data]

    fig, ax1 = plt.subplots()
    ind = np.arange(len(keys))  # the x locations for the groups
    width = 0.35  # the width of the bars
    # add some text for labels, title and axes ticks
    ax1.set_title('Time/Energy Data for Configuration ' + config)
    ax1.set_xticks(ind + width)
    ax1.set_xticklabels(keys, rotation=45)
    fig.set_tight_layout(True)
    fig.set_size_inches(len(plot_data) / 1.5, 8)

    ax2 = ax1.twinx()

    # Normalize
    if normalize:
        total_times_std /= np.sum(total_times)
        total_times /= np.sum(total_times)
        total_energies_std /= np.sum(total_energies)
        total_energies /= np.sum(total_energies)
        ax1.set_ylabel('Time (Normalized)')
        ax2.set_ylabel('Energy (Normalized)')
    else:
        # set time in us instead of ns
        total_times_std /= np.array(1000000.0)
        total_times /= np.array(1000000.0)
        total_energies_std /= np.array(1000000.0)
        total_energies /= np.array(1000000.0)
        ax1.set_ylabel('Time (ms)')
        ax2.set_ylabel('Energy (Joules)')

    rects1 = ax1.bar(ind, total_times, width, color='r', yerr=total_times_std)
    rects2 = ax2.bar(ind + width, total_energies, width, color='y', yerr=total_energies_std)
    ax1.legend([rects1[0], rects2[0]], ['Time', 'Energy'])

    # set axis
    x1, x2, y1, y2 = plt.axis()
    if normalize:
        ax1.set_ylim(ymin=0, ymax=1)
        ax2.set_ylim(ymin=0, ymax=1)
    else:
        ax1.set_ylim(ymin=0, ymax=((max_time + max_time_std) * 1.25 / 1000000.0))
        ax2.set_ylim(ymin=0, ymax=((max_energy + max_energy_std) * 1.25 / 1000000.0))

    autolabel(rects1, ax1)
    autolabel(rects2, ax2)

    # plt.show()
    plt.savefig(path.join(output_dir, config + ".png"))
    plt.close(fig)


def create_raw_total_data(config_data):
    """Get the raw data to plot for a configuration
    Return: [(profiler, time_mean, time_stddev, energy_mean, energy_stddev)]

    Keyword arguments:
    config_data -- (trial, trial_data)
    """
    # We can't assume that the same number of heartbeats are always issued across trials
    # key: profiler name; value: list of timing sums for each trial
    profiler_total_times = {}
    # key: profiler name; value: list of energy sums for each trial
    profiler_total_energies = {}
    for (t, td) in config_data:
        for (profiler, ts, te, es, ee) in td:
            # sum the total times and energies for each profiler in this trial
            total_time = np.sum(te - ts)
            total_energy = np.sum(ee - es)
            # add to list to be averaged later
            time_list = profiler_total_times.get(profiler, [])
            time_list.append(total_time)
            profiler_total_times[profiler] = time_list
            energy_list = profiler_total_energies.get(profiler, [])
            energy_list.append(total_energy)
            profiler_total_energies[profiler] = energy_list

    # Get mean and stddev for time and energy totals
    return [(profiler,
             np.mean(profiler_total_times[profiler]),
             np.std(profiler_total_times[profiler]),
             np.mean(profiler_total_energies[profiler]),
             np.std(profiler_total_energies[profiler]))
            for profiler in profiler_total_times.keys()]


def plot_all_raw_totals(config_list, output_dir):
    """Plot column charts of the raw total time/energy spent in each profiler category.

    Keyword arguments:
    config_list -- [(config, result of process_config_dir(...))]
    output_dir -- where to write plots to
    """
    raw_total_norm_out_dir = path.join(output_dir, 'raw_totals_normalized')
    os.makedirs(raw_total_norm_out_dir)
    raw_total_out_dir = path.join(output_dir, 'raw_totals')
    os.makedirs(raw_total_out_dir)

    # (name, (profiler, (time_mean, time_stddev, energy_mean, energy_stddev)))
    raw_totals_data = [(config, create_raw_total_data(config_data)) for (config, config_data) in config_list]

    mean_times = []
    mean_times_std = []
    mean_energies = []
    mean_energies_std = []
    for profiler_tup in [config_tup[1] for config_tup in raw_totals_data]:
        for (p, tt, tts, te, tes) in profiler_tup:
            mean_times.append(tt)
            mean_times_std.append(tts)
            mean_energies.append(te)
            mean_energies_std.append(tes)
    # get consistent max time/energy values across plots
    max_t = np.max(mean_times)
    max_t_std = np.max(mean_times_std)
    max_e = np.max(mean_energies)
    max_e_std = np.max(mean_energies_std)
    [plot_raw_totals(data[0], data[1], max_t, max_t_std, max_e, max_e_std, raw_total_norm_out_dir, True)
        for data in raw_totals_data]
    [plot_raw_totals(data[0], data[1], max_t, max_t_std, max_e, max_e_std, raw_total_out_dir, False)
        for data in raw_totals_data]


def plot_trial_time_series(config, trial, trial_data, max_end_time, max_power, output_dir):
    """Plot time series for a single trial.

    Keyword arguments:
    config -- the config name
    trial -- the trial name
    trial_data -- [(profiler, [start times], [end times], [start energies], [end energies])]
    max_end_time -- single value to use as max X axis value (for consistency across trials)
    output_dir -- the output directory
    """
    # TODO: Some profilers may have parallel tasks - need to identify this on plots
    max_end_time = max_end_time / 1000000.0
    trial_data = sorted(trial_data)
    fig, ax1 = plt.subplots()
    keys = [p for (p, ts, te, es, ee) in trial_data]
    # add some text for labels, title and axes ticks
    ax1.set_title('Profiler Activity for ' + config + ', ' + trial)
    ax1.set_xlabel('Time (ms)')
    ax1.grid(True)
    width = 8  # the width of the bars
    ax1.set_yticks(10 * np.arange(1, len(keys) + 2))
    ax1.set_yticklabels(keys)
    ax1.set_ylim(ymin=0, ymax=((len(trial_data) + 1) * 10))
    ax1.set_xlim(xmin=0, xmax=max_end_time)
    fig.set_tight_layout(True)
    fig.set_size_inches(16, len(trial_data) / 3)

    i = 10
    for (p, ts, te, es, ee) in trial_data:
        xranges = [(ts[j] / 1000000.0, (te[j] - ts[j]) / 1000000.0) for j in xrange(len(ts))]
        ax1.broken_barh(xranges, (i - 0.5 * width, width))
        i += 10
    # place a vbar at the final time for this trial
    last_profiler_times = map(np.nanmax, filter(lambda x: len(x) > 0, [te for (p, ts, te, es, ee) in trial_data]))
    plt.axvline(np.max(last_profiler_times) / 1000000.0, color='black')

    power_times = []
    power_values = []
    for (p, ts, te, es, ee) in trial_data:
        if p == ENERGY_PROFILER_NAME:
            power_times = te / 1000000.0
            power_values = (ee - es) / ((te - ts) / 1000.0)
    ax2 = ax1.twinx()
    ax2.set_xlim(xmin=0, xmax=max_end_time)
    ax2.set_ylim(ymin=0, ymax=max_power)
    ax2.set_ylabel('Power (Watts)')
    ax2.plot(power_times, power_values, color='r')

    # plt.show()
    plt.savefig(path.join(output_dir, "ts_" + config + "_" + trial + ".png"))
    plt.close(fig)


def hb_energy_times_to_power(es, ee, ts, te):
    """Compute power from start and end energy and times.
    Return: power values
    """
    return (ee - es) / ((te - ts) / 1000.0)


def plot_all_time_series(config_list, output_dir):
    """Plot column charts of the raw total time/energy spent in each profiler category.

    Keyword arguments:
    config_list -- [(config, result of process_config_dir(...))]
    output_dir -- where to write plots to
    """
    time_series_out_dir = path.join(output_dir, 'time_series')
    os.makedirs(time_series_out_dir)

    max_end_times = []
    max_power_values = []
    for (c, cd) in config_list:
        for (t, td) in cd:
            trial_max_end_times = map(np.nanmax, filter(lambda x: len(x) > 0, [te for (p, ts, te, es, ee) in td]))
            max_end_times.append(np.nanmax(trial_max_end_times))
            for (p, ts, te, es, ee) in td:
                # We only care about the energy profiler (others aren't reliable for instant power anyway)
                if p == ENERGY_PROFILER_NAME and len(te) > 0:
                    max_power_values.append(np.nanmax(hb_energy_times_to_power(es, ee, ts, te)))
    max_time = np.nanmax(max_end_times)
    max_power = np.nanmax(np.array(max_power_values)) * 1.2  # leave a little space at the top

    for (config, config_data) in config_list:
        [plot_trial_time_series(config, trial, trial_data, max_time, max_power, time_series_out_dir)
            for (trial, trial_data) in config_data]


def read_heartbeat_log(profiler_hb_log):
    """Read a heartbeat log file.
    Return: (profiler name, [start times], [end times], [start energies], [end energies], [instant powers])

    Keyword arguments:
    profiler_hb_log -- the file to read
    """
    with warnings.catch_warnings():
        try:
            warnings.simplefilter("ignore")
            time_start, time_end, energy_start, energy_end = \
                np.loadtxt(profiler_hb_log,
                           dtype=np.dtype('uint64'),
                           skiprows=1,
                           usecols=(HB_LOG_IDX_START_TIME,
                                    HB_LOG_IDX_END_TIME,
                                    HB_LOG_IDX_START_ENERGY,
                                    HB_LOG_IDX_END_ENERGY),
                           unpack=True,
                           ndmin=1)
        except ValueError:
            time_start, time_end, energy_start, energy_end = [], [], [], []
    name = path.split(profiler_hb_log)[1].split('-')[1].split('.')[0]
    return (name,
            np.atleast_1d(time_start),
            np.atleast_1d(time_end),
            np.atleast_1d(energy_start),
            np.atleast_1d(energy_end))


def process_trial_dir(trial_dir):
    """Process trial directory.
    Return: [(profiler name, [start times], [end times], [start energies], [end energies])]
    Time and energy are normalized to 0 start values.

    Keyword arguments:
    trial_dir -- the directory for this trial
    """
    log_data = map(lambda h: read_heartbeat_log(path.join(trial_dir, h)),
                   filter(lambda f: f.endswith(".log"), os.listdir(trial_dir)))

    # Find the earliest timestamps and energy readings
    min_t = np.nanmin(map(np.nanmin, filter(lambda x: len(x) > 0, [ts for (profiler, ts, te, es, ee) in log_data])))
    min_e = np.nanmin(map(np.nanmin, filter(lambda x: len(x) > 0, [es for (profiler, ts, te, es, ee) in log_data])))

    # Normalize timing/energy data to start values of 0
    return [(profiler, ts - min_t, te - min_t, es - min_e, ee - min_e) for (profiler, ts, te, es, ee) in log_data]


def process_config_dir(config_dir):
    """Process a configuration directory.
    Return: [(trial, [(profiler name, [start times], [end times], [start energies], [end energies])])]

    Keyword arguments:
    config_dir -- the directory for this configuration - contains subdirectories for each trial
    """
    return [(trial_dir, process_trial_dir(path.join(config_dir, trial_dir))) for trial_dir in os.listdir(config_dir)]


def process_logs(log_dir):
    """Process log directory.
    Return: [(config, [(trial, [(profiler name, [start times], [end times], [start energies], [end energies])])])]

    Keyword arguments:
    log_dir -- the log directory to process - contains subdirectories for each configuration
    """
    return [((config_dir.split('_')[1], process_config_dir(path.join(log_dir, config_dir))))
            for config_dir in os.listdir(log_dir)]


def find_best_executions(log_dir):
    """Get the best time, energy, and power from the characterization summaries.
    Return: ((config, trial, min_time), (config, trial, min_energy), (config, trial, min_power))

    Keyword arguments:
    results -- the results from process_logs(...).
    """
    DEFAULT = ('', '', 1000000000.0)
    min_time = DEFAULT
    min_energy = DEFAULT
    min_power = DEFAULT
    for config_dir in os.listdir(log_dir):
        for trial_dir in os.listdir(path.join(log_dir, config_dir)):
            with open(path.join(log_dir, config_dir, trial_dir, SUMMARY_OUTPUT), "r") as s:
                lines = s.readlines()
                time = float(lines[SUMMARY_TIME_IDX].split(':')[1])
                energy = int(lines[SUMMARY_ENERGY_IDX].split(':')[1])
                power = float(lines[SUMMARY_POWER_IDX].split(':')[1])
                if time < min_time[2]:
                    min_time = (config_dir, trial_dir, time)
                if energy < min_energy[2]:
                    min_energy = (config_dir, trial_dir, energy)
                if power < min_power:
                    min_power = (config_dir, trial_dir, power)
    return (min_time, min_energy, min_power)


def main():
    """This script processes the log files from the "characterize.py" script and produces visualizations.
    """
    # Default log directory
    directory = 'heartbeat_logs'
    # Default output directory
    output_dir = 'plots'

    # Parsing the input of the script
    parser = argparse.ArgumentParser(description="Process Heartbeat log files from characterization")
    parser.add_argument("-d", "--directory",
                        default=directory,
                        help="Heartbeat log directory \"-d heartbeat_logs\"")
    parser.add_argument("-o", "--output",
                        default=output_dir,
                        help="Specify the log output directory, for example \"-o plots\"")

    args = parser.parse_args()
    if args.directory:
        directory = args.directory
    if args.output:
        output_dir = args.output

    if not os.path.exists(directory):
        print "Input directory does not exist: " + directory
        sys.exit(1)

    if os.path.exists(output_dir):
        print "Output directory already exists: " + output_dir
        sys.exit(1)

    res = process_logs(directory)

    best = find_best_executions(directory)
    print 'Best time:', best[0]
    print 'Best energy:', best[1]
    print 'Best power:', best[2]

    os.makedirs(output_dir)
    plot_all_raw_totals(res, output_dir)
    plot_all_time_series(res, output_dir)

if __name__ == "__main__":
    main()

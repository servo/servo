This crate hosts the Servo profiler.
Its APIs can be found in the `profile_traits` crate.


# Heartbeats

Heartbeats allow fine-grained timing and energy profiling of Servo tasks specified in the `ProfilerCategory` enum (see the `profile_traits::time` module).
When enabled, a heartbeat is issued for each profiler category event.
They also compute the average performance and power for three levels of granularity:

* Global: the entire runtime.
* Window: the category's last `N` events, where `N` is the size of a sliding window.
* Instant: the category's most recent event.

## Enabling

Heartbeats are enabled for categories by setting proper environment variables prior to launching Servo.

For each desired category, set the `SERVO_HEARTBEAT_ENABLE_MyCategory` environment variable to any value (an empty string will do) where `MyCategory` is the `ProfilerCategory` name exactly as it appears in the enum.
For example:

```
SERVO_HEARTBEAT_ENABLE_LayoutPerform=""
```

Then set the `SERVO_HEARTBEAT_LOG_MyCategory` environment variable so Servo knows where to write the results.
For example:

```
SERVO_HEARTBEAT_LOG_LayoutPerform="/tmp/heartbeat-LayoutPerform.log"
```

The target directory must already exist and be writeable.
Results are written to the log file every `N` heartbeats and when the profiler shuts down.

You can optionally specify the size of the sliding window by setting `SERVO_HEARTBEAT_WINDOW_MyCategory` to a positive integer value.
The default value is `20`.
For example:

```
SERVO_HEARTBEAT_WINDOW_LayoutPerform=20
```

The window size is also how many heartbeats will be stored in memory.

## Log Files

Log files are whitespace-delimited.

`HB` is the heartbeat number, ordered by when they are registered (not necessarily start or end time!).
The count starts at `0`.

`Tag` is a client-specified identifier for each heartbeat.
Servo does not use this, so the value is always `0`.

`Work` is the amount of work completed for a particular heartbeat and is used in computing performance.
At this time, Servo simply specifies `1` unit of work for each heartbeat.

`Time` and `Energy` have `Start` and `End` values as captured during runtime.
Time is measured in nanoseconds and energy is measured in microjoules.

`Work`, `Time`, and `Energy` also have `Global` and `Window` values which are the summed over the entire runtime and sliding window period, respectively.

`Perf` (performance) and `Pwr` (power) have `Global`, `Window`, and `Instant` values as described above.


# Energy Profiling

Energy monitoring is hardware and platform-specific, so it is only enabled with the `energy-profiling` feature.

To use energy profiling, you must have a compatible `energymon-default` implementation installed to your system as `energymon-default-static` when building Servo.
Otherwise a default dummy implementation is used.
The library is linked through a chain of dependencies:

* servo::profile_traits
  * energymon - Rust abstractions
    * energymon-default-sys - Rust bindings to `energymon-default.h`
      * energymon-default-static: A statically linked C library installed to the system that implements `energymon.h` and `energymon-default.h`

For instructions on building existing native libraries, visit the [energymon project source](https://github.com/energymon/energymon).
You may also write your own implementation of `energymon.h` and `energymon-default.h` and install it as `energymon-default-static` where pkg-config can find it.

Once you install the proper library, you will need to rebuild the `energymon-default-sys` crate.
The most straightforward way to do this is to do a clean build of Servo.

To build Servo with the `energy-profiling` feature enabled, pass `--features "energy-profiling"` to the `mach` command, e.g.:

```sh
./mach build -r --features "energy-profiling"
```

When running Servo, you will want to enable the desired Heartbeats to record the results.

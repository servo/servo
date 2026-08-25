/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Helper to boost critical threads
//!
//! On heterogeneous (e.g. big.LITTLE) CPUs the scheduler might not be able to
//! determine which threads are critical.
//! This module serves as an entry point to define policies for thread-affinity,
//! thread priority and core frequency boosting or other mechanisms a platform
//! might have for apps to allow the OS to optimize thread performance.
//!
//! TODO(#46813): We provide a default implementation here, but the embedder
//! should be able to customize this (e.g. if they care more about energy efficiency
//! then raw performance, for example in battery saving mode).
//! TODO: For android we should look at the performance hint API in the NDK.
//! TODO: For mobile linux, the ohos implementation could be shared, however other APIs
//!    like uclamp or enhanced thread priorities might work better there?

use servo_config::pref;

#[cfg(target_env = "ohos")]
mod platform {
    //! On `ohos` targets we only have the `OH_QoS_SetThreadQoS` API from qos/qos.h,
    //! which influences scheduling priority, but empirically does not help with ensuring
    //! important servo threads like script get scheduled on larger cores, presumably because
    //! we do a lot of IPC, and the workload doesn't pass heuristic thresholds to get promoted
    //! to a larger core.
    //! [uclamp_min](https://docs.kernel.org/scheduler/sched-util-clamp.html) is supported but
    //! ignored (empirically tested) on the hongmeng kernel.
    //! Thqt leaves thread affinity as a last fallback, which allows us to prevent scheduling a
    //! thread on little cores. Android developer docs discourages using thread affinity, since it
    //! will also negatively affect power consumption if the little cores would have been
    //! sufficient, but for now this is all we have (pending better official OH APIs, perhaps
    //! modeled after the android performance hint API).
    use std::fs;
    use std::sync::LazyLock;

    use super::{BoostAffinity, ThreadPriority};

    // Constants copied from `qos/qos.h`. Avoids depending on ohos-libqos-sys just for this one function.
    // See also <https://docs.rs/ohos-libqos-sys/0.1.0/src/ohos_libqos_sys/qos_ffi.rs.html#21>
    const QOS_USER_INITIATED: i32 = 3;
    const QOS_USER_INTERACTIVE: i32 = 5;

    #[link(name = "qos")]
    #[expect(unsafe_code)]
    unsafe extern "C" {
        // SAFETY: Calling this function is always safe.
        safe fn OH_QoS_SetThreadQoS(level: i32) -> i32;
    }

    // The current maximum supported by CPU_SET is 1024, so u16 is sufficiently large.
    // <https://man7.org/linux/man-pages/man3/CPU_SET.3.html>
    type CoreId = u16;

    static NON_LITTLE_CPU_CORES: LazyLock<Option<Box<[CoreId]>>> = LazyLock::new(|| {
        non_little_cpus()
            .inspect_err(|error| log::error!("Failed to determine non-little cpu cores: {error}"))
            .ok()?
            .map(|cores| cores.into_boxed_slice())
    });

    fn parse_cpu_list() -> Result<Vec<CoreId>, String> {
        let cpu_possible_file = "/sys/devices/system/cpu/possible";
        let list = fs::read_to_string(cpu_possible_file)
            .map_err(|error| format!("failed to read {cpu_possible_file}: {error:?}"))?;

        let mut cpus = Vec::new();
        // See <https://docs.kernel.org/admin-guide/cputopology.html> / cpulist_parse
        for part in list.trim().split(',') {
            if let Some((a, b)) = part.split_once('-') {
                if let (Ok(a), Ok(b)) = (a.trim().parse::<CoreId>(), b.trim().parse::<CoreId>()) {
                    cpus.extend(a..=b);
                }
            } else if let Ok(core_id) = part.trim().parse::<CoreId>() {
                cpus.push(core_id);
            } else {
                log::warn!("Unexpected CPU line: {part:?} in {cpu_possible_file}.");
            }
        }
        Ok(cpus)
    }

    /// Per-cpu relative capacity (normalized to 1024 for the strongest core in a system)
    ///
    /// <https://www.kernel.org/doc/Documentation/devicetree/bindings/arm/cpu-capacity.txt>
    fn capacity_of(cpu: CoreId) -> Result<u64, String> {
        let cpu_capacity_file = format!("/sys/devices/system/cpu/cpu{cpu}/cpu_capacity");
        fs::read_to_string(cpu_capacity_file)
            .map_err(|error| error.to_string())?
            .trim()
            .parse::<u64>()
            .map_err(|error| error.to_string())
    }

    /// Determine the CPU ids of cores not in the little class.
    ///
    /// Returns `Err` on internal / parsing errors.
    /// Returns `Ok(None)` if the cpu is not heterogeneous, or if there would only be a single
    /// cpu in the non-little group.
    fn non_little_cpus() -> Result<Option<Vec<CoreId>>, String> {
        let cpus = parse_cpu_list()?;
        let cpu_and_caps: Vec<(CoreId, u64)> = cpus
            .iter()
            .map(|&cpu| capacity_of(cpu).map(|cap| (cpu, cap)))
            .collect::<Result<_, _>>()?;

        let mut classes: Vec<u64> = cpu_and_caps.iter().map(|&(_, cap)| cap).collect();
        classes.sort_unstable();
        classes.dedup();
        if classes.len() < 2 {
            return Ok(None);
        }
        let little = classes[0];
        let chosen: Vec<CoreId> = cpu_and_caps
            .iter()
            .filter(|&&(_, cap)| cap > little)
            .map(|&(cpu, _)| cpu)
            .collect();
        if chosen.len() < 2 {
            return Ok(None);
        }
        Ok(Some(chosen))
    }

    #[expect(unsafe_code)]
    fn pin_thread_to_medium_or_large_cpus() -> Result<(), String> {
        // Note: If we encountered an error when parsing the cpu structure, then
        // we logged the error in the LazyLock (once), which avoids flooding the logs
        // with error messages for every thread we want to pin.
        let Some(cpus) = NON_LITTLE_CPU_CORES.as_ref() else {
            return Ok(());
        };
        let ret = unsafe {
            let mut set: libc::cpu_set_t = std::mem::zeroed();
            for &cpu in cpus {
                libc::CPU_SET(cpu.into(), &mut set);
            }
            libc::sched_setaffinity(0, size_of::<libc::cpu_set_t>(), &set)
        };
        if ret != 0 {
            return Err(format!(
                "Failed to set thread affinity: {:?}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
    }

    pub fn boost_thread(priority: ThreadPriority, boost_affinity: BoostAffinity) {
        let qos_rc = match priority {
            ThreadPriority::Elevated => OH_QoS_SetThreadQoS(QOS_USER_INITIATED),
            ThreadPriority::Critical => OH_QoS_SetThreadQoS(QOS_USER_INTERACTIVE),
            ThreadPriority::Default => 0,
        };
        if qos_rc != 0 {
            log::warn!("Failed to boost thread priority. `OH_QoS_SetThreadQoS` returned {qos_rc}");
        }
        if matches!(boost_affinity, BoostAffinity::Boost) &&
            let Err(error) = pin_thread_to_medium_or_large_cpus()
        {
            log::warn!(
                "Failed to pin {} to medium or large cpus: {error:?}",
                std::thread::current().name().unwrap_or("<unnamed>"),
            );
        }
    }
}

#[cfg(not(target_env = "ohos"))]
mod platform {
    pub fn boost_thread(_: super::ThreadPriority, _: super::BoostAffinity) {}
}

pub enum ThreadPriority {
    /// Priority will remain unchanged.
    Default,
    /// Increase the thread priority.
    Elevated,
    /// Higher priority than `Elevated`, should be used sparingly.
    Critical,
}

/// On heterougenous systems (e.g. big.LITTLE architecture), select
/// whether we should attempt to boost this thread to a larger core.
/// The exact effect is platform specific, a hint and may be ignored.
pub enum BoostAffinity {
    No,
    /// Prioritize Medium or Large cores and avoid small cores.
    Boost,
}

/// Hint to the scheduler that this thread should be prioritised.
///
/// No effect if `pref!(perf_thread_boost_enabled)` is `false`.
///
/// TODO: The exact API and inner-workings are subject to change:
/// - This is a hint to servo / the embedder and can be a no-op.
/// - We might want to pass a thread identifier (enum variant?) so that we
///   (or the embedder) can customize the optimization based on the thread
///   without relying on parsing the thread-name.
/// - Some optimizations like thread affinity selection also affect children threads,
///   if spawned after this call, so placement can be important.
#[allow(unsafe_code)]
pub fn boost_thread(priority: ThreadPriority, boost_affinity: BoostAffinity) {
    if pref!(perf_thread_boost_enabled) {
        platform::boost_thread(priority, boost_affinity)
    }
}

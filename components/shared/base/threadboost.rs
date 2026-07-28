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
//! should be able to customize this (e.g. if they care more about energery efficiency
//! then raw performance, for example in battery saving mode).
//! TODO: For android we should look at the performance hint API in the NDK.
//! TODO: For mobile linux, the ohos implementation could be shared, however other APIs
//!    like uclamp or enhanced thread priorities might work better there?

#[cfg(target_env = "ohos")]
mod ohos {
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

    /// Highest QoS level from OHOS `qos/qos.h` (API 12+).
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
            .inspect_err(|err| log::error!("Failed to determine non-little cpu cores: {}", err))
            .ok()?
            .map(|cores| cores.into_boxed_slice())
    });

    fn parse_cpu_list() -> Result<Vec<CoreId>, String> {
        let cpu_possible_file = "/sys/devices/system/cpu/possible";
        let list = fs::read_to_string(cpu_possible_file)
            .map_err(|e| format!("failed to read {cpu_possible_file}: {e:?}"))?;

        let mut cpus = Vec::new();
        // See <https://docs.kernel.org/admin-guide/cputopology.html> / cpulist_parse
        for part in list.trim().split(',') {
            if let Some((a, b)) = part.split_once('-') {
                if let (Ok(a), Ok(b)) = (a.trim().parse::<CoreId>(), b.trim().parse::<CoreId>()) {
                    cpus.extend(a..=b);
                }
            } else if let Ok(c) = part.trim().parse::<CoreId>() {
                cpus.push(c);
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
            .map_err(|e| e.to_string())?
            .trim()
            .parse::<u64>()
            .map_err(|e| e.to_string())
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
            .map(|&(c, _)| c)
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
            for &c in cpus {
                libc::CPU_SET(c.into(), &mut set);
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

    pub fn mark_thread_as_critical() {
        let qos_rc = OH_QoS_SetThreadQoS(QOS_USER_INTERACTIVE);
        if qos_rc != 0 {
            log::warn!("Failed to set QOS_USER_INTERACTIVE");
        }
        if let Err(error) = pin_thread_to_medium_or_large_cpus() {
            log::warn!(
                "Failed to pin {} to medium or large cpus: {:?}",
                std::thread::current().name().unwrap_or("<unnamed>"),
                error
            );
        }
    }
}

/// Hint to the scheduler that this thread should be prioritised.
///
/// TODO: The exact API and inner-workings are subject to change:
/// - This is a hint to servo / the embedder and can be a no-op.
/// - We might want to pass a thread identifier (enum variant?) so that we
///   (or the embedder) can customize the optimization based on the thread
///   without relying on parsing the thread-name.
/// - Some optimizations like thread affinity selection also affect children threads,
///   if spawned after this call, so placement can be important.
#[allow(unsafe_code)]
pub fn mark_thread_as_critical() {
    #[cfg(target_env = "ohos")]
    ohos::mark_thread_as_critical()
}

//! The GPU probe and monitor, through NVML.
//!
//! `nvml-wrapper` loads `libnvidia-ml.so` at run time, so a machine without
//! the NVIDIA driver gets a `Missing` GPU instead of a binary that fails to
//! start.

use std::collections::HashSet;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use nvml_wrapper::bitmasks::device::ThrottleReasons;
use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};
use nvml_wrapper::enums::device::UsedGpuMemory;
use nvml_wrapper::error::{Bits, NvmlError};
use nvml_wrapper::{Device, Nvml, cuda_driver_version_major, cuda_driver_version_minor};

use crate::application::ports::{GpuMonitor, GpuProbe, Monitoring, Readings};
use crate::domain::bench::{GpuReading, Scenario, Slowdown};
use crate::domain::machine::{Gpu, GpuProcess, Probed, Version};
use crate::domain::text::escape_controls;

/// Longest process name NVML is asked for.
const PROCESS_NAME_LEN: usize = 256;
/// NVML's clock-event reasons for a policy of the board, BoardLimit, or of
/// the driver, Reliability (`nvml.h` of CUDA 13.4), which nvml-wrapper
/// 0.13's `ThrottleReasons` lacks.
const POLICY_REASONS: u64 = 0x200 | 0x400;

/// The process's NVML, initialised by the first read of the GPU, so a
/// command that never reads it never loads the library. The first read,
/// which loads the library, initialises NVML and attaches the device, costs
/// tens of milliseconds; a later read, about a tenth of one. An
/// initialisation that failed stays failed for the process. NVML is never shut down: nvmlShutdown is
/// skipped at exit, which leaves the driver nothing to clean, and NVML opens
/// its descriptors close-on-exec, so the gateway `up` starts inherits none.
fn nvml() -> Result<&'static Nvml, &'static NvmlError> {
    static NVML: OnceLock<Result<Nvml, NvmlError>> = OnceLock::new();
    NVML.get_or_init(Nvml::init).as_ref()
}

/// Reads the first NVIDIA GPU.
pub struct NvmlProbe;

impl GpuProbe for NvmlProbe {
    fn gpu(&self) -> Probed<Gpu> {
        let nvml = match nvml() {
            Ok(nvml) => nvml,
            Err(err) => return init_failure(err),
        };
        match read_gpu(nvml) {
            Ok(Some(gpu)) => Probed::Found(gpu),
            Ok(None) => Probed::Failed("NVML sees no GPU".to_owned()),
            Err(err) => Probed::Failed(err.to_string()),
        }
    }
}

/// A library that cannot be loaded means no driver; any other error means a
/// driver that does not answer.
fn init_failure(err: &NvmlError) -> Probed<Gpu> {
    match err {
        NvmlError::LibloadingError(_) | NvmlError::LibraryNotFound => Probed::Missing,
        other => Probed::Failed(other.to_string()),
    }
}

fn read_gpu(nvml: &Nvml) -> Result<Option<Gpu>, NvmlError> {
    if nvml.device_count()? == 0 {
        return Ok(None);
    }
    let device = nvml.device_by_index(0)?;
    let memory = device.memory_info()?;
    let capability = device.cuda_compute_capability()?;
    let cuda = nvml.sys_cuda_driver_version()?;

    let mut seen = HashSet::new();
    let mut processes = Vec::new();
    let running = device
        .running_compute_processes()?
        .into_iter()
        .chain(device.running_graphics_processes()?);
    for info in running {
        if !seen.insert(info.pid) {
            continue;
        }
        let name = nvml
            .sys_process_name(info.pid, PROCESS_NAME_LEN)
            .map(|name| process_name(&name))
            .unwrap_or_else(|_| "unknown".to_owned());
        let memory = match info.used_gpu_memory {
            UsedGpuMemory::Used(bytes) => Some(bytes),
            UsedGpuMemory::Unavailable => None,
        };
        processes.push(GpuProcess {
            pid: info.pid,
            name,
            memory,
        });
    }

    Ok(Some(Gpu {
        name: device.name()?,
        compute_capability: (
            non_negative(capability.major),
            non_negative(capability.minor),
        ),
        memory_total: memory.total,
        memory_reserved: memory.reserved,
        memory_used: memory.used,
        driver_version: nvml.sys_driver_version()?,
        cuda_version: Version::new(
            non_negative(cuda_driver_version_major(cuda)),
            non_negative(cuda_driver_version_minor(cuda)),
            0,
        ),
        processes,
    }))
}

/// Reads the first NVIDIA GPU on a thread of its own, at a fixed period,
/// through the process's NVML.
pub struct NvmlMonitor;

impl GpuMonitor for NvmlMonitor {
    fn start(&self, period: Duration) -> Result<Box<dyn Monitoring + '_>> {
        let (stop, stopped) = mpsc::channel();
        let (ready, started) = mpsc::channel();
        let scenario = Arc::new(Mutex::new(None));
        let label = Arc::clone(&scenario);
        let thread = thread::Builder::new()
            .name("gpu-monitor".to_owned())
            .spawn(move || monitor(period, &label, &stopped, &ready))
            .context("cannot start reading the GPU")?;
        match started.recv() {
            Ok(Ok(())) => Ok(Box::new(NvmlMonitoring {
                stop: Some(stop),
                scenario,
                thread: Some(thread),
            })),
            Ok(Err(err)) => {
                let _ = thread.join();
                Err(anyhow!(err).context("cannot read the GPU (NVML)"))
            }
            Err(_) => bail!("the GPU's reading ended before it started"),
        }
    }
}

/// The GPU being read by [`NvmlMonitor`].
struct NvmlMonitoring {
    /// Dropped, it stops the reading thread.
    stop: Option<Sender<()>>,
    scenario: Arc<Mutex<Option<Scenario>>>,
    thread: Option<JoinHandle<Readings>>,
}

impl Monitoring for NvmlMonitoring {
    fn enter(&self, scenario: Scenario) {
        if let Ok(mut label) = self.scenario.lock() {
            *label = Some(scenario);
        }
    }

    fn finish(mut self: Box<Self>) -> Result<Readings> {
        self.stop.take();
        self.thread
            .take()
            .context("the GPU's reading has already stopped")?
            .join()
            .map_err(|_| anyhow!("the GPU's reading failed"))
    }
}

impl Drop for NvmlMonitoring {
    fn drop(&mut self) {
        self.stop.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Reads the GPU every `period` until `stopped` says so or its sender is
/// gone, each reading labelled with the scenario `label` holds then. Tells
/// `ready` whether a first reading worked; a later one that fails is
/// counted, with its error.
fn monitor(
    period: Duration,
    label: &Mutex<Option<Scenario>>,
    stopped: &Receiver<()>,
    ready: &Sender<Result<(), String>>,
) -> Readings {
    let scenario = || label.lock().ok().and_then(|label| *label);
    let nvml = match nvml() {
        Ok(nvml) => nvml,
        Err(err) => {
            let _ = ready.send(Err(err.to_string()));
            return Readings::default();
        }
    };
    let first = nvml.device_by_index(0).and_then(|device| {
        let reading = read(&device, scenario())?;
        Ok((device, reading))
    });
    let (device, first) = match first {
        Ok(first) => first,
        Err(err) => {
            let _ = ready.send(Err(err.to_string()));
            return Readings::default();
        }
    };
    let _ = ready.send(Ok(()));
    let mut readings = Readings {
        taken: vec![first],
        failed: None,
    };
    while let Err(RecvTimeoutError::Timeout) = stopped.recv_timeout(period) {
        match read(&device, scenario()) {
            Ok(reading) => readings.taken.push(reading),
            Err(err) => {
                let count = readings.failed.as_ref().map_or(0, |(count, _)| *count);
                readings.failed = Some((count + 1, err.to_string()));
            }
        }
    }
    readings
}

fn read(device: &Device, scenario: Option<Scenario>) -> Result<GpuReading, NvmlError> {
    let clock_events = clock_events(device)?;
    Ok(GpuReading {
        scenario,
        memory_used_mib: device.memory_info()?.used >> 20,
        temperature_c: device.temperature(TemperatureSensor::Gpu)?,
        sm_clock_mhz: device.clock_info(Clock::SM)?,
        power_w: f64::from(device.power_usage()?) / 1000.0,
        slowdown: slowdown(clock_events),
        clock_events: Some(clock_events),
    })
}

/// NVML's clock-event reasons, every bit kept: nvml-wrapper's
/// `current_throttle_reasons` drops the bits its flags lack, and its strict
/// form returns them as an error.
fn clock_events(device: &Device) -> Result<u64, NvmlError> {
    match device.current_throttle_reasons_strict() {
        Ok(reasons) => Ok(reasons.bits()),
        Err(NvmlError::IncorrectBits(Bits::U64(mask))) => Ok(mask),
        Err(err) => Err(err),
    }
}

/// What NVML's clock-event reasons, `mask`, say slowed the clocks down.
fn slowdown(mask: u64) -> Slowdown {
    let reasons = ThrottleReasons::from_bits_truncate(mask);
    Slowdown {
        power: reasons.contains(ThrottleReasons::SW_POWER_CAP),
        thermal: reasons.intersects(
            ThrottleReasons::SW_THERMAL_SLOWDOWN | ThrottleReasons::HW_THERMAL_SLOWDOWN,
        ),
        hardware: reasons
            .intersects(ThrottleReasons::HW_SLOWDOWN | ThrottleReasons::HW_POWER_BRAKE_SLOWDOWN),
        policy: mask & POLICY_REASONS != 0,
    }
}

fn non_negative(value: i32) -> u32 {
    u32::try_from(value).unwrap_or(0)
}

/// A process's name as NVML gives it, made safe for a terminal: the
/// program name, with its control and invisible format characters escaped.
/// Whoever runs the process chooses the name, other users included.
fn process_name(raw: &str) -> String {
    escape_controls(&base_name(raw))
}

/// NVML may return a full path: keep the program name.
fn base_name(name: &str) -> String {
    Path::new(name).file_name().map_or_else(
        || name.to_owned(),
        |base| base.to_string_lossy().into_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_process_name_keeps_its_program_and_escapes_its_controls() {
        assert_eq!(
            process_name("/x/\u{1b}[2K\u{1b}[1Gok    Idle VRAM fake\u{1b}[0m"),
            "\\u001B[2K\\u001B[1Gok    Idle VRAM fake\\u001B[0m"
        );
        assert_eq!(process_name("/usr/bin/gnome-shell"), "gnome-shell");
    }

    #[test]
    fn a_machine_without_nvml_has_no_gpu() {
        assert_eq!(init_failure(&NvmlError::LibraryNotFound), Probed::Missing);
    }

    #[test]
    fn a_driver_that_does_not_answer_is_a_failure() {
        assert_eq!(
            init_failure(&NvmlError::DriverNotLoaded),
            Probed::Failed(NvmlError::DriverNotLoaded.to_string())
        );
        assert!(matches!(
            init_failure(&NvmlError::NoPermission),
            Probed::Failed(_)
        ));
    }

    #[test]
    fn process_names_lose_their_directory() {
        assert_eq!(base_name("/usr/bin/gnome-shell"), "gnome-shell");
        assert_eq!(base_name("Xwayland"), "Xwayland");
    }

    #[test]
    fn throttle_reasons_are_told_apart_by_cause() {
        assert_eq!(slowdown(0), Slowdown::default());
        assert_eq!(
            slowdown((ThrottleReasons::SW_POWER_CAP | ThrottleReasons::GPU_IDLE).bits()),
            Slowdown {
                power: true,
                ..Slowdown::default()
            }
        );
        for thermal in [
            ThrottleReasons::SW_THERMAL_SLOWDOWN,
            ThrottleReasons::HW_THERMAL_SLOWDOWN,
        ] {
            assert!(slowdown(thermal.bits()).thermal);
        }
        let braked = slowdown(ThrottleReasons::HW_POWER_BRAKE_SLOWDOWN.bits());
        assert!(braked.hardware && !braked.thermal && !braked.power);
        assert!(slowdown(ThrottleReasons::HW_SLOWDOWN.bits()).hardware);
        // Reliability, which this machine's GPU reports at idle
        // (`nvidia-smi --query-gpu=clocks_throttle_reasons.active`), and
        // BoardLimit: bits nvml-wrapper's flags lack.
        assert_eq!(
            slowdown(0x400),
            Slowdown {
                policy: true,
                ..Slowdown::default()
            }
        );
        assert!(slowdown(0x200).policy);
        let capped = slowdown(ThrottleReasons::SW_POWER_CAP.bits() | 0x400);
        assert!(capped.power && capped.policy && !capped.thermal);
    }

    #[test]
    fn negative_values_become_zero() {
        assert_eq!(non_negative(-1), 0);
        assert_eq!(non_negative(12), 12);
    }
}

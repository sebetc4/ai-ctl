# Build Toolchain

What native builds need on Fedora with an NVIDIA GPU, how to install it on a fresh machine, how to
change the CUDA version or the Fedora release without breaking the driver, and the record of what
was run on this machine. ai-ctl never installs anything: `ai-ctl doctor` checks every point below
and prints these commands as remedies.

---

## 1. Requirements

| Component | Requirement | Package | Why |
|---|---|---|---|
| NVIDIA driver | Any recent driver; its CUDA version caps the toolkit | `akmod-nvidia`, `xorg-x11-drv-nvidia-cuda` (RPM Fusion) | The driver's CUDA version, shown by `doctor` and `nvidia-smi`, is the newest toolkit it can run |
| CUDA toolkit | 12.8 or newer, not 13.2, not newer than the driver's CUDA version | `cuda-toolkit-<major>-<minor>` (NVIDIA) | 12.8 adds the `120a-real` architecture; 13.2 is reported to produce wrong output in llama.cpp builds (llama.cpp issue #21255) |
| Host compiler | A GCC the toolkit accepts | `gcc-c++`, or `gcc<N>-c++` when the default is too new | `nvcc` refuses a GCC newer than the one its `crt/host_config.h` names; 13.2 stops at GCC 15, 13.4 at GCC 16 |
| CMake | 3.18 or newer | `cmake` | ggml's CUDA backend requires it |
| git | 2.19.2 or newer | `git` | Builds use a partial clone (`--filter=blob:none`) whose checkout fetches the files lazily: 2.19.0 made `clone` check the objects it receives, and 2.19.2 fixed the lazy fetch (git's release notes) |
| Ninja | Optional | `ninja-build` | Builds use it when it is installed: an exact progress line, and better scheduling of the CUDA compilations than Make |
| ccache | Optional | `ccache` | ggml compiles through it when it is installed (`GGML_CCACHE`), which speeds up a rebuild; builds never use its `g++` wrapper as a compiler |

ai-ctl itself builds with the Rust toolchain its `rust-toolchain.toml` pins, here and in the CI:
rustup installs it, with `clippy` and `rustfmt`, at the first `cargo` or `rustc` command run in
the repository. `doctor` does not check it.

---

## 2. Fresh Install (Fedora 44)

**1. Driver, from RPM Fusion.** Enable the repositories, then, with Secure Boot on, create the
akmods signing key and enroll it before installing the driver:

```bash
sudo dnf install \
  https://mirrors.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm \
  https://mirrors.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm

# Secure Boot only: reboot after `mokutil` and enroll the key in the MOK manager
sudo dnf install kmodtool akmods mokutil openssl
sudo kmodgenca -a
sudo mokutil --import /etc/pki/akmods/certs/public_key.der
systemctl reboot

sudo dnf install akmod-nvidia xorg-x11-drv-nvidia-cuda
modinfo -F version nvidia   # wait until it prints the version, then reboot
```

**2. NVIDIA's CUDA repository, with its driver packages excluded.** The repository carries a whole
driver stack of its own (`nvidia-driver*`, `nvidia-settings`, `kmod-nvidia-*-dkms`, …), at the same
epoch as RPM Fusion's. Without the exclusion, the first driver NVIDIA publishes before RPM Fusion
replaces part of the driver with packages that no longer match the kernel module.

```bash
sudo dnf config-manager addrepo --from-repofile=https://developer.download.nvidia.com/compute/cuda/repos/fedora44/x86_64/cuda-fedora44.repo
sudo dnf config-manager setopt cuda-fedora44-x86_64.exclude='nvidia-*,kmod-nvidia*,xorg-x11-*nvidia*,libnvidia-*,dnf-plugin-nvidia'
dnf repoquery --repo cuda-fedora44-x86_64 'nvidia-*'   # must print nothing
```

**3. CUDA toolkit, by its versioned package.** Pick the driver's CUDA version (13.4 for driver
615.78.08). Never install the `cuda-toolkit` or `cuda-toolkit-<major>` meta-packages: the first
follows NVIDIA's newest release, the second the newest of its major version, and a `dnf upgrade`
would then install a toolkit the driver cannot run. `cuda-toolkit-<major>-<minor>` stays on its
version. Since CUDA 13.4, no meta-package pulls in a driver.

```bash
sudo dnf install cuda-toolkit-13-4
echo 'export PATH=/usr/local/cuda/bin:$PATH' >> ~/.bashrc   # for interactive use
```

`/usr/local/cuda` is an `alternatives` link whose priority is the version (13.4 → 134), so it
points at the newest toolkit installed without another step. `doctor` finds `nvcc` on the `PATH`,
and at `/usr/local/cuda/bin/nvcc` otherwise.

**4. Build tools.**

```bash
sudo dnf install gcc-c++ cmake git ninja-build
```

When `doctor` reports that `nvcc` does not accept the default GCC, install the newest one it
accepts beside it, `gcc15-c++` for instance; `doctor` and the builder pick it by themselves.

**5. Check.** `ai-ctl doctor` must show the Toolchain section without a failure. Its toolkit
remedy gives steps 2 and 3 for the running Fedora release, leaving out the repository or the
exclusion when they are already there; for a toolkit newer than the driver, it then removes the
newer one as §3 does, so that `/usr/local/cuda` falls back to the driver's. It warns about an enabled CUDA repository that offers
driver packages, and about an unversioned `cuda-toolkit` or `cuda-toolkit-<major>` package. The
repository check reads dnf's cache only, so it is current after the last `dnf` command that
refreshed the metadata.

---

## 3. Changing the CUDA Version

Toolkits install side by side under `/usr/local/cuda-<major>.<minor>`, so a new one is installed
and checked before the old one goes.

```bash
ai-ctl doctor                      # the driver's CUDA version caps the target
sudo dnf install cuda-toolkit-13-4 # the new toolkit; /usr/local/cuda follows it
ai-ctl doctor                      # Toolchain: CUDA toolkit and host compiler ok
```

To go back to an older toolkit still installed, pin both links, then return to automatic mode:

```bash
sudo alternatives --set cuda /usr/local/cuda-13.3
sudo alternatives --set cuda-13 /usr/local/cuda-13.3
sudo alternatives --auto cuda && sudo alternatives --auto cuda-13
```

To remove the old toolkit, remove its meta-package, then the packages of that version it leaves
behind, which `dnf` keeps when nothing marks them as dependencies:

```bash
sudo dnf remove cuda-toolkit-13-2
rpm -qa --qf '%{name}\n' | grep -E -- '-13-2(-|$)'   # what is left of 13.2
sudo dnf remove $(rpm -qa --qf '%{name}\n' | grep -E -- '-13-2(-|$)')
```

Before confirming a removal, read the transaction: it lists only packages of the old version. A
driver package in it (`akmod-nvidia`, `xorg-x11-drv-nvidia-*`) means something is wrong: answer no.

A build loads the CUDA libraries through `/usr/local/cuda` by their major version
(`libcudart.so.13`, `libcublas.so.13`): it keeps running after a minor change, and breaks when no
toolkit of its major version is left.

---

## 4. Changing the Fedora Release

NVIDIA's repository names the Fedora release in its URL instead of following `$releasever`, and
NVIDIA publishes the repository of a new release weeks or months after it. After
`dnf system-upgrade`, the previous release's repository stays configured and keeps working. Once
the new one exists, switch to it, carrying the exclusion over:

```bash
curl -fsI https://developer.download.nvidia.com/compute/cuda/repos/fedora45/x86_64/cuda-fedora45.repo   # exists?
sudo dnf config-manager addrepo --from-repofile=https://developer.download.nvidia.com/compute/cuda/repos/fedora45/x86_64/cuda-fedora45.repo
sudo dnf config-manager setopt cuda-fedora45-x86_64.exclude='nvidia-*,kmod-nvidia*,xorg-x11-*nvidia*,libnvidia-*,dnf-plugin-nvidia'
sudo dnf config-manager setopt cuda-fedora44-x86_64.enabled=0
```

A new release may also bring a GCC newer than the toolkit accepts: `ai-ctl doctor` says so in its
host compiler line, and section 2, step 4 applies.

---

## 5. Record of This Machine

Newest first. Every change to the driver, the toolkit, the compilers or their repositories, and
to the cargo tools the reviews run, is recorded here with the commands run.

### 2026-11-06 — Rust 1.97.1 → 1.99.1, pinned by the repository; rustup 1.29.0 → 1.29.3

Before: rustup 1.29.0, installed from rustup.rs in `~/.cargo/bin`, its `stable` toolchain at
1.97.1 of 2026-07-14, the default; no toolchain file in the repository. rustup's newest stable was
1.99.1, of 2026-10-13.

```bash
rustup show active-toolchain   # in the repository, rust-toolchain.toml written
rustup toolchain install       # the toolchain the file names
```

After: `1.99.1-x86_64-unknown-linux-gnu`, its minimal profile with `clippy` and `rustfmt`, active
in the repository by `rust-toolchain.toml`: rustc 1.99.1 (58cf8f9c3 2026-10-13), cargo 1.99.1,
clippy 0.1.99, rustfmt 1.10.0. `rustup show active-toolchain` installed it, rustup installing a
toolchain file's toolchain at its first use; `rustup toolchain install` then found it installed
and updated rustup itself to 1.29.3, of 2026-11-02, rustup's self-update being on. `stable` stays
at 1.97.1, the default outside the repository. The cargo tools in `~/.cargo/bin` were not
touched; in the repository they run on 1.99.1.

### 2026-11-02 — cargo-deny 0.20.2

Before: no `cargo-deny`; `cargo-audit` 0.22.2 and `cargo-mutants` 27.1.0 in `~/.cargo/bin`,
installed on 2026-10-13, before this record held the cargo tools. rustc 1.97.1.

```bash
cargo install --locked cargo-deny@0.20.2
```

After: `cargo-deny` 0.20.2 in `~/.cargo/bin`, built in 58 s with the versions its own
`Cargo.lock` pins: crates.io's newest release, of 2026-07-09, which needs Rust 1.88. The
`reviewing-delivered-code` skill's dependency axis runs `cargo deny check` under `deny.toml`.

### 2026-10-15 — Driver 615.71.09 → 615.78.08

Before: driver 615.71.09 from RPM Fusion, CUDA 13.4 toolkit (`nvcc` 13.4.92) from
`cuda-fedora44`.

The update came from GNOME Software, which applies packages through PackageKit and
`dnf5daemon-server`: no command was typed. `dnf history` holds the transactions (times in UTC):

| ID | Time | What |
|---|---|---|
| 341 | 2026-10-15 10:49 | `dnf5daemon-server`: `akmod-nvidia` and `xorg-x11-drv-nvidia-cuda`, with their eight NVIDIA dependencies, 615.71.09 → 615.78.08 from `rpmfusion-nonfree-updates` |
| 342 | 2026-10-15 10:51 | akmods: `kmod-nvidia` 615.78.08 for kernel `7.2.10-200.audio.fc44` |
| 344 | 2026-10-16 08:45 | akmods: `kmod-nvidia` 615.78.08 for kernel `7.2.10-200.fc44` |

`dnf history info 341` lists the packages.

After: driver and kernel module 615.78.08, whose CUDA version is still 13.4 (`nvidia-smi`: CUDA
UMD 13.4). The toolkit was not touched: `nvcc` 13.4.92. `nvidia-cdi-refresh` rewrote
`/var/run/cdi/nvidia.yaml`, which now mounts `libcuda.so.615.78.08`.

### 2026-10-09 — CUDA 13.2 → 13.4

Before: Fedora 44, driver 615.71.09 from RPM Fusion (CUDA 13.4), the unversioned `cuda-toolkit`
meta-package at 13.2.2 from `cuda-fedora43`, whose newest toolkit is 13.2; that repository excluded
the driver packages. `nvcc` 13.2 accepted GCC 15 at most, so builds used `g++-15` (`gcc15-c++`).

```bash
sudo dnf config-manager addrepo --from-repofile=https://developer.download.nvidia.com/compute/cuda/repos/fedora44/x86_64/cuda-fedora44.repo
sudo dnf install cuda-toolkit-13-4
sudo dnf remove cuda-toolkit cuda-toolkit-13-2
sudo dnf config-manager setopt cuda-fedora43-x86_64.enabled=0
sudo dnf upgrade
sudo dnf remove cuda-opencl-13-2 cuda-driver-devel-13-2 cuda-toolkit-13-2-config-common
sudo dnf config-manager setopt cuda-fedora44-x86_64.exclude='nvidia-*,kmod-nvidia*,xorg-x11-*nvidia*,libnvidia-*,dnf-plugin-nvidia'
```

The `dnf` transactions are in `dnf history` (330 to 333); the `config-manager` calls are not, and
their place in this order is approximate.

After: `nvcc` 13.4.92 under `/usr/local/cuda-13.4`, no 13.2 package left, `cuda-fedora43`
disabled, `cuda-fedora44` without driver packages. `nvcc` 13.4 accepts GCC 16, so `doctor` picks
the default `g++` 16.2.1. The driver was not touched: `dnf history` shows no driver package in
these transactions.

---

## Sources

- [NVIDIA — CUDA Installation Guide for Linux, Fedora](https://docs.nvidia.com/cuda/cuda-installation-guide-linux/#fedora)
- [RPM Fusion — Configuration](https://rpmfusion.org/Configuration)
- [RPM Fusion — NVIDIA howto](https://rpmfusion.org/Howto/NVIDIA)
- [RPM Fusion — Secure Boot](https://rpmfusion.org/Howto/Secure%20Boot)
- [llama.cpp issue #21255 — CUDA 13.2 and IQ3_S](https://github.com/ggml-org/llama.cpp/issues/21255)
- [llama.cpp — CUDA architectures](https://github.com/ggml-org/llama.cpp/blob/master/ggml/src/ggml-cuda/CMakeLists.txt)

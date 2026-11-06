//! The gateway as a process: llama-swap started detached from the terminal,
//! and the files it leaves in the state directory — its record, the
//! configuration it runs, and its log. Beside it, the `/proc` table that
//! places the GPU's processes, and the check that its address is free.
//!
//! The record and the configuration are written to a hidden file first and
//! renamed into place, so a reader never finds half of one. Whether a
//! recorded gateway still runs is read from `/proc/<pid>/exe`: a PID the
//! kernel gave to another program since is not the gateway.

use std::any::Any;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::net::{IpAddr, Ipv6Addr, SocketAddr, TcpListener};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::application::ports::{
    GatewayProcess, GatewayRecord, Liveness, PortCheck, ProcessStat, ProcessTable, Signal,
    StartedGateway,
};
use crate::domain::gateway_config::{API_KEY_VARIABLE, ConfigFile};

use super::api_key::KeyFile;
use super::command;
use super::files::{self, create_private_dir, lock, replace_file};

const RECORD: &str = "gateway.json";
const CONFIG: &str = "llama-swap.yaml";
const LOG: &str = "gateway.log";
/// The log of the previous start, kept beside the new one.
const PREVIOUS_LOG: &str = "gateway.log.1";

/// The gateway's files: its record and configuration in `dir`, its log in
/// `logs_dir`; and the file of the API key it is started with.
pub struct LocalGateway {
    dir: PathBuf,
    logs_dir: PathBuf,
    key: KeyFile,
}

impl LocalGateway {
    pub fn new(dir: PathBuf, logs_dir: PathBuf, key: KeyFile) -> Self {
        Self { dir, logs_dir, key }
    }

    fn record_file(&self) -> PathBuf {
        self.dir.join(RECORD)
    }

    fn config_file(&self) -> PathBuf {
        self.dir.join(CONFIG)
    }

    /// Writes `text` to `name` in the gateway's directory, by a rename.
    fn write(&self, name: &str, text: &str) -> Result<()> {
        create_private_dir(&self.dir)?;
        replace_file(&self.dir.join(name), text)
    }

    /// Writes a file a model reads, by a rename, creating its directory; the
    /// configuration puts each such file under [`GatewayProcess::files_dir`].
    fn write_file(&self, file: &ConfigFile) -> Result<()> {
        let path = &file.path;
        let parent = path
            .parent()
            .ok_or_else(|| anyhow!("{} has no directory", path.display()))?;
        create_private_dir(parent)?;
        replace_file(path, &file.content)
    }

    /// Opens a new log, keeping the previous one as `gateway.log.1`.
    fn new_log(&self) -> Result<File> {
        create_private_dir(&self.logs_dir)?;
        let log = self.log();
        match fs::rename(&log, self.logs_dir.join(PREVIOUS_LOG)) {
            Err(err) if err.kind() != io::ErrorKind::NotFound => {
                return Err(err).with_context(|| format!("cannot rotate {}", log.display()));
            }
            _ => {}
        }
        File::create(&log).with_context(|| format!("cannot create {}", log.display()))
    }
}

/// The record as `gateway.json` holds it.
#[derive(Serialize, Deserialize)]
struct RecordFile {
    pid: u32,
    binary: PathBuf,
    listen: SocketAddr,
}

/// Reads the program a process runs from its `/proc/<pid>/exe` link, which
/// names `binary` resolved, or `binary (deleted)` once the file is removed.
/// The record keeps `binary` as given, which the lifecycle compares with
/// the binary it would start.
fn liveness_of(pid: u32, binary: &Path) -> Result<Liveness> {
    let link = PathBuf::from(format!("/proc/{pid}/exe"));
    match fs::read_link(&link) {
        Ok(target) => {
            let binary = resolved(binary);
            let deleted = format!("{} (deleted)", binary.display());
            if target == binary || target.as_os_str() == deleted.as_str() {
                Ok(Liveness::Running)
            } else {
                Ok(Liveness::Reused(Some(target)))
            }
        }
        // A process of another user hides its program.
        Err(err) if err.kind() == io::ErrorKind::PermissionDenied => Ok(Liveness::Reused(None)),
        // No process, or a zombie, which has no program any more.
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Liveness::Exited),
        Err(err) => Err(err).with_context(|| format!("cannot read {}", link.display())),
    }
}

/// `path` with every link resolved, as the kernel names a program. The
/// part of a removed file's path that remains is resolved, and the rest
/// kept as it was: the kernel resolved it when the program started.
fn resolved(path: &Path) -> PathBuf {
    let mut removed = Vec::new();
    let mut existing = path;
    loop {
        if let Ok(real) = fs::canonicalize(existing) {
            return removed
                .iter()
                .rev()
                .fold(real, |real, name| real.join(name));
        }
        match (existing.parent(), existing.file_name()) {
            (Some(parent), Some(name)) => {
                removed.push(name);
                existing = parent;
            }
            _ => return path.to_owned(),
        }
    }
}

impl GatewayProcess for LocalGateway {
    fn lock(&self, doing: &str) -> Result<Box<dyn Any>> {
        create_private_dir(&self.dir)?;
        lock(&self.dir, doing)
    }

    fn record(&self) -> Result<Option<GatewayRecord>> {
        let path = self.record_file();
        let Some(text) = files::read_if_present(&path)? else {
            return Ok(None);
        };
        let record: RecordFile = serde_json::from_str(&text).with_context(|| {
            format!(
                "{} is malformed; remove it once no llama-swap runs",
                path.display()
            )
        })?;
        Ok(Some(GatewayRecord {
            pid: record.pid,
            binary: record.binary,
            listen: record.listen,
        }))
    }

    fn liveness(&self, record: &GatewayRecord) -> Result<Liveness> {
        liveness_of(record.pid, &record.binary)
    }

    fn forget(&self) -> Result<()> {
        let path = self.record_file();
        match fs::remove_file(&path) {
            Err(err) if err.kind() != io::ErrorKind::NotFound => {
                Err(err).with_context(|| format!("cannot remove {}", path.display()))
            }
            _ => Ok(()),
        }
    }

    fn config(&self) -> Result<Option<String>> {
        let path = self.config_file();
        files::read_if_present(&path)
    }

    fn files_dir(&self) -> &Path {
        &self.dir
    }

    fn start(
        &self,
        binary: &Path,
        config: &str,
        files: &[ConfigFile],
        listen: SocketAddr,
    ) -> Result<Box<dyn StartedGateway + '_>> {
        let key = self.key.read_or_create()?;
        for file in files {
            self.write_file(file)?;
        }
        self.write(CONFIG, config)?;
        let log = self.new_log()?;
        let output = log
            .try_clone()
            .with_context(|| format!("cannot open {}", self.log().display()))?;
        let mut command = Command::new(binary);
        command
            .arg("-config")
            .arg(self.config_file())
            .arg("-listen")
            .arg(listen.to_string())
            .current_dir(&self.dir)
            // The configuration reads the key from there, and passes it on
            // to each backend in its own environment.
            .env(API_KEY_VARIABLE, key)
            .stdin(Stdio::null())
            .stdout(output)
            .stderr(log)
            // Out of the terminal's process group: Ctrl-C and the end of the
            // session do not reach it.
            .process_group(0);
        let mut child = command::spawn(&mut command)
            .with_context(|| format!("cannot run {}", binary.display()))?;
        let record = RecordFile {
            pid: child.id(),
            binary: binary.to_owned(),
            listen,
        };
        let recorded = serde_json::to_string_pretty(&record)
            .context("cannot serialize the gateway's record")
            .and_then(|text| self.write(RECORD, &format!("{text}\n")));
        if let Err(err) = recorded {
            // Unrecorded, the gateway could not be stopped by ai-ctl.
            let _ = child.kill();
            let _ = child.wait();
            return Err(err);
        }
        Ok(Box::new(Started(child)))
    }

    fn signal(&self, pid: u32, signal: Signal) -> Result<()> {
        let name = match signal {
            Signal::Terminate => "TERM",
            Signal::Kill => "KILL",
        };
        let pid = pid.to_string();
        let missing = "kill not found; run `sudo dnf install util-linux`";
        let output = command::checked(command::run("kill", &["-s", name, &pid]), missing)?;
        if output.success || output.stderr.contains("No such process") {
            return Ok(());
        }
        bail!("kill -s {name} {pid}: {}", output.failure())
    }

    fn log(&self) -> PathBuf {
        self.logs_dir.join(LOG)
    }

    fn log_tail(&self, lines: usize) -> Result<String> {
        let log = self.log();
        let bytes = fs::read(&log).with_context(|| format!("cannot read {}", log.display()))?;
        let text = String::from_utf8_lossy(&bytes);
        let all: Vec<&str> = text.lines().collect();
        Ok(all[all.len().saturating_sub(lines)..].join("\n"))
    }

    fn read_log(&self, offset: u64, max: usize) -> Result<Option<Vec<u8>>> {
        let log = self.log();
        let mut file = match File::open(&log) {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err).with_context(|| format!("cannot open {}", log.display())),
        };
        let mut read = || -> io::Result<Option<Vec<u8>>> {
            if file.metadata()?.len() < offset {
                return Ok(None);
            }
            file.seek(SeekFrom::Start(offset))?;
            let mut bytes = Vec::new();
            (&mut file).take(max as u64).read_to_end(&mut bytes)?;
            Ok(Some(bytes))
        };
        read().with_context(|| format!("cannot read {}", log.display()))
    }
}

/// The processes of this machine, read from `/proc`.
pub struct Procfs;

impl ProcessTable for Procfs {
    fn stat(&self, pid: u32) -> Result<Option<ProcessStat>> {
        let path = PathBuf::from(format!("/proc/{pid}/stat"));
        let Some(stat) = files::read_if_present(&path)? else {
            return Ok(None);
        };
        process_stat(&stat)
            .map(Some)
            .with_context(|| format!("unexpected {}: {}", path.display(), stat.trim()))
    }

    fn in_container(&self, pid: u32) -> Result<bool> {
        let path = PathBuf::from(format!("/proc/{pid}/cgroup"));
        Ok(files::read_if_present(&path)?.is_some_and(|cgroup| names_a_container(&cgroup)))
    }

    fn program(&self, pid: u32) -> Result<Option<PathBuf>> {
        let link = PathBuf::from(format!("/proc/{pid}/exe"));
        match fs::read_link(&link) {
            Ok(target) => Ok(Some(target)),
            Err(err)
                if matches!(
                    err.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
                ) =>
            {
                Ok(None)
            }
            Err(err) => Err(err).with_context(|| format!("cannot read {}", link.display())),
        }
    }
}

/// Whether a port is free, tried by binding it.
pub struct TcpPorts;

impl PortCheck for TcpPorts {
    /// Tried on each socket llama-swap would bind, the dual-stack one
    /// included: a program on `[::1]` holds the port of `0.0.0.0` for it.
    fn is_free(&self, listen: SocketAddr) -> Result<bool> {
        let listener = go_listener(listen);
        match TcpListener::bind(listener.address) {
            Ok(_) => {}
            Err(err) if err.kind() == io::ErrorKind::AddrInUse => return Ok(false),
            Err(err) => return Err(err).with_context(|| format!("cannot listen on {listen}")),
        }
        // Any other failure is a kernel without IPv6, where Go listens on the
        // IPv4 address alone.
        let taken = listener.dual_stack.is_some_and(|dual_stack| {
            TcpListener::bind(dual_stack).is_err_and(|err| err.kind() == io::ErrorKind::AddrInUse)
        });
        Ok(!taken)
    }

    /// Whether one of `pid`'s descriptors is a socket that the kernel's
    /// tables show listening on `listen`, or on the dual-stack socket Go
    /// opens for it. A process that is gone holds nothing.
    fn listens(&self, pid: u32, listen: SocketAddr) -> Result<bool> {
        let listener = go_listener(listen);
        let mut inodes = Vec::new();
        for address in std::iter::once(listener.address).chain(listener.dual_stack) {
            // A kernel without IPv6 has no `tcp6` table.
            if let Some(text) = files::read_if_present(socket_table(address))? {
                inodes.extend(listening_inodes(&text, address));
            }
        }
        if inodes.is_empty() {
            return Ok(false);
        }
        let dir = PathBuf::from(format!("/proc/{pid}/fd"));
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(err) => return Err(err).with_context(|| format!("cannot read {}", dir.display())),
        };
        let mut links = Vec::new();
        for entry in entries {
            let path = entry
                .with_context(|| format!("cannot read {}", dir.display()))?
                .path();
            let link = fs::read_link(&path);
            links.push((path, link));
        }
        holds_socket(links, &inodes)
    }
}

/// Whether one of a process's descriptors, each read as its link, is a
/// socket of `inodes`.
fn holds_socket(links: Vec<(PathBuf, io::Result<PathBuf>)>, inodes: &[u64]) -> Result<bool> {
    for (path, link) in links {
        match link {
            Ok(target) if socket_inode(&target).is_some_and(|inode| inodes.contains(&inode)) => {
                return Ok(true);
            }
            Ok(_) => {}
            // A descriptor closed since the listing names nothing.
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(err).with_context(|| format!("cannot read {}", path.display())),
        }
    }
    Ok(false)
}

/// The sockets llama-swap, a Go program, listens on for `listen`: the
/// address, canonical, and for an unspecified IPv4 address the dual-stack
/// `[::]` that Go opens in its place when the kernel has IPv6.
#[derive(Debug, PartialEq, Eq)]
struct GoListener {
    address: SocketAddr,
    dual_stack: Option<SocketAddr>,
}

fn go_listener(listen: SocketAddr) -> GoListener {
    let address = SocketAddr::new(listen.ip().to_canonical(), listen.port());
    let dual_stack = match address.ip() {
        IpAddr::V4(ip) if ip.is_unspecified() => {
            Some(SocketAddr::from((Ipv6Addr::UNSPECIFIED, address.port())))
        }
        _ => None,
    };
    GoListener {
        address,
        dual_stack,
    }
}

/// The table the kernel lists `address`'s sockets in.
fn socket_table(address: SocketAddr) -> &'static Path {
    Path::new(if address.is_ipv4() {
        "/proc/net/tcp"
    } else {
        "/proc/net/tcp6"
    })
}

/// The inodes of the sockets that a `/proc/net/tcp` or `/proc/net/tcp6`
/// table shows listening on `listen`: state `0A`, and the local address as
/// the kernel prints it, each 32-bit word of the IP in the machine's byte
/// order, then the port, in upper-case hexadecimal.
fn listening_inodes(table: &str, listen: SocketAddr) -> Vec<u64> {
    let local = kernel_address(listen);
    table
        .lines()
        .skip(1)
        .filter_map(|row| {
            let fields: Vec<&str> = row.split_whitespace().collect();
            let listening = fields.get(1) == Some(&local.as_str()) && fields.get(3) == Some(&"0A");
            listening.then(|| fields.get(9)?.parse().ok()).flatten()
        })
        .collect()
}

fn kernel_address(addr: SocketAddr) -> String {
    let octets = match addr.ip() {
        IpAddr::V4(ip) => ip.octets().to_vec(),
        IpAddr::V6(ip) => ip.octets().to_vec(),
    };
    let (words, _) = octets.as_chunks::<4>();
    let ip: String = words
        .iter()
        .map(|word| format!("{:08X}", u32::from_ne_bytes(*word)))
        .collect();
    format!("{ip}:{:04X}", addr.port())
}

/// The inode in a descriptor's link, `socket:[<inode>]`.
fn socket_inode(target: &Path) -> Option<u64> {
    target
        .to_str()?
        .strip_prefix("socket:[")?
        .strip_suffix(']')?
        .parse()
        .ok()
}

/// The program name and the parent in a `/proc/<pid>/stat` line: the name
/// is in parentheses and may hold any character, the parent is the second
/// field after it.
fn process_stat(stat: &str) -> Option<ProcessStat> {
    let (before, after) = stat.rsplit_once(')')?;
    let (_, name) = before.split_once('(')?;
    let parent = after.split_whitespace().nth(1)?.parse().ok()?;
    let name = name.to_owned();
    Some(ProcessStat { name, parent })
}

/// Whether a `/proc/<pid>/cgroup` file names the scope of a podman
/// container, `libpod-<id>.scope`: a process of a container, even one
/// whose conmon is gone.
fn names_a_container(cgroup: &str) -> bool {
    cgroup
        .lines()
        .any(|line| line.split('/').any(|part| part.starts_with("libpod-")))
}

/// llama-swap, started by this process.
struct Started(Child);

impl StartedGateway for Started {
    fn pid(&self) -> u32 {
        self.0.id()
    }

    fn exited(&mut self) -> Result<Option<String>> {
        let status = self.0.try_wait().context("cannot wait for llama-swap")?;
        Ok(status.map(|status| status.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::files;
    use std::os::unix::fs::PermissionsExt;

    fn gateway(root: &Path) -> LocalGateway {
        LocalGateway::new(
            root.join("gateway"),
            root.join("logs"),
            KeyFile::new(root.join("api-key")),
        )
    }

    fn record(pid: u32, binary: &Path) -> GatewayRecord {
        GatewayRecord {
            pid,
            binary: binary.to_owned(),
            listen: "127.0.0.1:8080".parse().unwrap(),
        }
    }

    #[test]
    fn this_process_runs_its_own_binary() {
        let me = std::env::current_exe().unwrap();
        assert_eq!(
            liveness_of(std::process::id(), &me).unwrap(),
            Liveness::Running
        );
        assert_eq!(
            liveness_of(std::process::id(), Path::new("/usr/bin/llama-swap")).unwrap(),
            Liveness::Reused(Some(me))
        );
    }

    #[test]
    fn an_exited_process_is_seen_as_such() {
        let mut child = command::spawn(&mut Command::new("true")).unwrap();
        let pid = child.id();
        child.wait().unwrap();
        assert_eq!(
            liveness_of(pid, Path::new("/usr/bin/true")).unwrap(),
            Liveness::Exited
        );
    }

    #[test]
    fn a_gateway_run_through_a_linked_directory_runs_until_it_exits() {
        // A data directory reached through a link, such as one moved to
        // another disk: the kernel names the program by its real path.
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        let release = real.join("llama-swap/v263");
        fs::create_dir_all(&release).unwrap();
        {
            let _writing = command::exclusive();
            fs::copy("/usr/bin/sleep", release.join("llama-swap")).unwrap();
        }
        let data = dir.path().join("data");
        std::os::unix::fs::symlink(&real, &data).unwrap();
        let binary = data.join("llama-swap/v263/llama-swap");

        let mut child = command::spawn(Command::new(&binary).arg("60")).unwrap();
        let pid = child.id();
        let installed = liveness_of(pid, &binary).unwrap();
        fs::remove_file(release.join("llama-swap")).unwrap();
        let file_removed = liveness_of(pid, &binary).unwrap();
        fs::remove_dir(&release).unwrap();
        let directory_removed = liveness_of(pid, &binary).unwrap();
        child.kill().unwrap();
        child.wait().unwrap();

        assert_eq!(
            (installed, file_removed, directory_removed),
            (Liveness::Running, Liveness::Running, Liveness::Running)
        );
        assert_eq!(liveness_of(pid, &binary).unwrap(), Liveness::Exited);
    }

    #[test]
    fn a_process_names_its_program_until_it_is_gone() {
        assert_eq!(
            Procfs.program(std::process::id()).unwrap(),
            Some(std::env::current_exe().unwrap())
        );
        let mut child = command::spawn(&mut Command::new("true")).unwrap();
        let pid = child.id();
        child.wait().unwrap();
        assert_eq!(Procfs.program(pid).unwrap(), None);
    }

    #[test]
    fn the_gateways_directories_are_created_for_the_user_only() {
        let _exclusive = command::exclusive();
        let dir = tempfile::tempdir().unwrap();
        let (locked, written, logged) = (
            dir.path().join("locked"),
            dir.path().join("written"),
            dir.path().join("logged"),
        );
        drop(gateway(&locked).lock("starting the gateway").unwrap());
        gateway(&written).write(RECORD, "{}").unwrap();
        drop(gateway(&logged).new_log().unwrap());
        for created in [
            locked.clone(),
            locked.join("gateway"),
            written.clone(),
            written.join("gateway"),
            logged.clone(),
            logged.join("logs"),
        ] {
            assert_eq!(files::mode(&created), 0o700, "{}", created.display());
        }
    }

    #[test]
    fn the_log_is_read_from_an_offset_in_bounded_pieces() {
        let dir = tempfile::tempdir().unwrap();
        let gateway = gateway(dir.path());
        assert_eq!(gateway.read_log(0, 4).unwrap(), None);
        fs::create_dir_all(dir.path().join("logs")).unwrap();
        fs::write(gateway.log(), "line 1\nline 2\n").unwrap();
        assert_eq!(gateway.read_log(0, 4).unwrap().unwrap(), b"line");
        assert_eq!(gateway.read_log(7, 100).unwrap().unwrap(), b"line 2\n");
        assert_eq!(gateway.read_log(14, 100).unwrap().unwrap(), b"");
        // Shorter than the offset: another log took its place.
        assert_eq!(gateway.read_log(15, 100).unwrap(), None);
    }

    const STAT_CONMON: &str = include_str!("../../tests/fixtures/proc-stat-conmon.txt");
    const STAT_ODD_NAME: &str = include_str!("../../tests/fixtures/proc-stat-odd-name.txt");
    const CGROUP_CONTAINER: &str =
        include_str!("../../tests/fixtures/proc-cgroup-podman-container.txt");
    const CGROUP_GNOME_SHELL: &str =
        include_str!("../../tests/fixtures/proc-cgroup-gnome-shell.txt");

    #[test]
    fn a_stat_line_gives_the_program_and_its_parent() {
        let stat = |name: &str, parent| {
            Some(ProcessStat {
                name: name.into(),
                parent,
            })
        };
        assert_eq!(process_stat(STAT_CONMON), stat("conmon", 4571));
        // A program named `x) R 1 (y`.
        assert_eq!(process_stat(STAT_ODD_NAME), stat("x) R 1 (y", 89867));
        assert_eq!(process_stat("77 (truncated"), None);
    }

    #[test]
    fn a_libpod_scope_names_a_container() {
        assert!(names_a_container(CGROUP_CONTAINER));
        assert!(!names_a_container(CGROUP_GNOME_SHELL));
        assert!(!names_a_container(""));
    }

    #[test]
    fn a_child_of_this_process_names_it_as_its_parent() {
        let mut child = command::spawn(Command::new("sleep").arg("60")).unwrap();
        let stat = Procfs.stat(child.id());
        let contained = Procfs.in_container(child.id());
        child.kill().unwrap();
        child.wait().unwrap();
        assert_eq!(
            stat.unwrap(),
            Some(ProcessStat {
                name: "sleep".into(),
                parent: std::process::id()
            })
        );
        assert!(!contained.unwrap());
    }

    #[test]
    fn a_process_that_is_gone_has_no_stat_and_no_container() {
        assert_eq!(Procfs.stat(u32::MAX).unwrap(), None);
        assert!(!Procfs.in_container(u32::MAX).unwrap());
    }

    #[test]
    fn no_record_reads_as_none_and_forgetting_it_is_harmless() {
        let root = tempfile::tempdir().unwrap();
        let gateway = gateway(root.path());
        assert_eq!(gateway.record().unwrap(), None);
        assert_eq!(gateway.config().unwrap(), None);
        gateway.forget().unwrap();
    }

    #[test]
    fn a_malformed_record_is_an_error_naming_it() {
        let root = tempfile::tempdir().unwrap();
        let gateway = gateway(root.path());
        gateway.write(RECORD, "{\"pid\": \"x\"}").unwrap();
        let message = gateway.record().unwrap_err().to_string();
        assert!(message.ends_with("gateway.json is malformed; remove it once no llama-swap runs"));
    }

    #[test]
    fn a_taken_port_is_not_free() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        assert!(!TcpPorts.is_free(addr).unwrap());
        // A child forked meanwhile would hold the socket open.
        let _releasing = command::exclusive();
        drop(listener);
        assert!(TcpPorts.is_free(addr).unwrap());
    }

    #[test]
    fn the_socket_table_gives_the_listener_and_not_its_connections() {
        let table = include_str!("../../tests/fixtures/proc-net-tcp-excerpt.txt");
        let listen = "127.0.0.1:34545".parse().unwrap();
        assert_eq!(listening_inodes(table, listen), [5130254]);
        // The client's own port, in a connected socket's row.
        assert!(listening_inodes(table, "127.0.0.1:44906".parse().unwrap()).is_empty());
        let table = include_str!("../../tests/fixtures/proc-net-tcp6-excerpt.txt");
        assert_eq!(
            listening_inodes(table, "[::1]:53305".parse().unwrap()),
            [5130257]
        );
        assert!(listening_inodes(table, "[::1]:34545".parse().unwrap()).is_empty());
        let table = include_str!("../../tests/fixtures/proc-net-tcp6-dual-stack.txt");
        assert_eq!(
            listening_inodes(table, "[::]:19200".parse().unwrap()),
            [1948879]
        );
    }

    #[test]
    fn an_address_of_no_interface_is_an_error_not_a_taken_port() {
        // TEST-NET-1, which no interface of this machine holds.
        let listen = "192.0.2.1:8080".parse().unwrap();
        assert_eq!(
            format!("{:#}", TcpPorts.is_free(listen).unwrap_err()),
            "cannot listen on 192.0.2.1:8080: Cannot assign requested address (os error 99)"
        );
    }

    #[test]
    fn go_listens_on_a_wildcard_ipv4_address_through_a_dual_stack_socket() {
        let listener = |listen: &str| go_listener(listen.parse().unwrap());
        let one = |address: &str| GoListener {
            address: address.parse().unwrap(),
            dual_stack: None,
        };
        let wildcard = GoListener {
            address: "0.0.0.0:8080".parse().unwrap(),
            dual_stack: Some("[::]:8080".parse().unwrap()),
        };
        assert_eq!(listener("0.0.0.0:8080"), wildcard);
        assert_eq!(listener("[::ffff:0.0.0.0]:8080"), wildcard);
        assert_eq!(listener("[::ffff:127.0.0.1]:8080"), one("127.0.0.1:8080"));
        assert_eq!(listener("127.0.0.1:8080"), one("127.0.0.1:8080"));
        assert_eq!(listener("[::]:8080"), one("[::]:8080"));
        assert_eq!(listener("[::1]:8080"), one("[::1]:8080"));
    }

    #[test]
    fn a_dual_stack_socket_listens_on_the_wildcard_ipv4_address() {
        let listener = TcpListener::bind("[::]:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let me = std::process::id();
        assert!(
            TcpPorts
                .listens(me, SocketAddr::from(([0, 0, 0, 0], port)))
                .unwrap()
        );
    }

    /// A listener on `[::1]` whose port is free on 127.0.0.1: the kernel may
    /// give `[::1]` a port that another socket of the machine holds on
    /// 127.0.0.1, such as the local end of a client's connection.
    fn on_ipv6_alone() -> (TcpListener, u16) {
        for _ in 0..100 {
            let listener = TcpListener::bind("[::1]:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            if TcpListener::bind(("127.0.0.1", port)).is_ok() {
                return (listener, port);
            }
        }
        panic!("no port of [::1] was free on 127.0.0.1 in 100 tries");
    }

    #[test]
    fn a_port_taken_on_ipv6_is_not_free_for_the_wildcard_ipv4_address() {
        // `is_free` releases the sockets it binds, which a child forked
        // meanwhile would still hold.
        let _exclusive = command::exclusive();
        let (_listener, port) = on_ipv6_alone();
        assert!(
            !TcpPorts
                .is_free(SocketAddr::from(([0, 0, 0, 0], port)))
                .unwrap()
        );
        // Go binds 127.0.0.1 as IPv4 alone, which `[::1]` leaves free.
        assert!(
            TcpPorts
                .is_free(SocketAddr::from(([127, 0, 0, 1], port)))
                .unwrap()
        );
    }

    #[test]
    fn a_process_holds_the_sockets_it_listens_on_and_no_other() {
        let v4 = TcpListener::bind("127.0.0.1:0").unwrap();
        let v6 = TcpListener::bind("[::1]:0").unwrap();
        let me = std::process::id();
        assert!(TcpPorts.listens(me, v4.local_addr().unwrap()).unwrap());
        assert!(TcpPorts.listens(me, v6.local_addr().unwrap()).unwrap());
        let mut other = command::spawn(Command::new("sleep").arg("30")).unwrap();
        let held = TcpPorts.listens(other.id(), v4.local_addr().unwrap());
        other.kill().unwrap();
        other.wait().unwrap();
        assert!(!held.unwrap());
        let closed = v4.local_addr().unwrap();
        drop(v4);
        assert!(!TcpPorts.listens(me, closed).unwrap());
    }

    #[test]
    fn a_descriptor_closed_since_the_listing_names_nothing() {
        let fd = |n: u32| PathBuf::from(format!("/proc/7/fd/{n}"));
        let socket = |inode: u64| Ok(PathBuf::from(format!("socket:[{inode}]")));
        let closed = || Err(io::Error::from(io::ErrorKind::NotFound));
        assert!(holds_socket(vec![(fd(3), closed()), (fd(4), socket(42))], &[42]).unwrap());
        assert!(!holds_socket(vec![(fd(3), closed()), (fd(4), socket(41))], &[42]).unwrap());
        let denied = Err(io::Error::from(io::ErrorKind::PermissionDenied));
        assert_eq!(
            format!(
                "{:#}",
                holds_socket(vec![(fd(3), denied)], &[42]).unwrap_err()
            ),
            "cannot read /proc/7/fd/3: permission denied"
        );
    }

    #[test]
    fn a_gone_process_listens_on_nothing_and_an_unreadable_one_is_an_error() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        assert!(!TcpPorts.listens(u32::MAX, addr).unwrap());
        // init's descriptors are root's, unless the tests run as root.
        if fs::read_dir("/proc/1/fd").is_err() {
            let message = format!("{:#}", TcpPorts.listens(1, addr).unwrap_err());
            assert_eq!(
                message,
                "cannot read /proc/1/fd: Permission denied (os error 13)"
            );
        }
    }

    /// Writes an executable script into `dir`.
    /// The PIDs of the processes whose command line names `program`.
    fn running(program: &Path) -> Vec<u32> {
        let program = program.as_os_str().as_encoded_bytes();
        fs::read_dir("/proc")
            .unwrap()
            .flatten()
            .filter_map(|entry| entry.file_name().to_str()?.parse::<u32>().ok())
            .filter(|pid| {
                fs::read(format!("/proc/{pid}/cmdline"))
                    .is_ok_and(|cmdline| cmdline.split(|&b| b == 0).any(|arg| arg == program))
            })
            .collect()
    }

    #[test]
    fn a_gateway_whose_record_cannot_be_written_is_stopped() {
        let root = tempfile::tempdir().unwrap();
        let gateway = gateway(root.path());
        // It keeps its script's path as its name while it runs.
        let binary = root.path().join("llama-swap");
        command::script(&binary, "exec -a \"$0\" sleep 30");
        // A directory in the record's place: the rename onto it fails.
        let record = root.path().join("gateway").join(RECORD);
        fs::create_dir_all(record.join("x")).unwrap();
        let listen = "127.0.0.1:8080".parse().unwrap();
        let message = format!(
            "{:#}",
            gateway
                .start(&binary, "models: {}\n", &[], listen)
                .err()
                .unwrap()
        );
        let left = running(&binary);
        for pid in &left {
            let _ = Command::new("kill")
                .args(["-s", "KILL", &pid.to_string()])
                .status();
        }
        assert!(
            message.starts_with(&format!("cannot write {}", record.display())),
            "{message}"
        );
        assert!(left.is_empty(), "left running: {left:?}");
    }

    fn wait(started: &mut dyn StartedGateway) -> String {
        loop {
            if let Some(status) = started.exited().unwrap() {
                return status;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn start_writes_the_configuration_records_the_process_and_logs_its_output() {
        let root = tempfile::tempdir().unwrap();
        let gateway = gateway(root.path());
        // The working directory, the arguments, the process group (field 5
        // of its stat line), the key, then a line on stderr.
        let binary = root.path().join("llama-swap");
        command::script(
            &binary,
            "pwd; echo \"$@\"; echo \"group $(cut -d' ' -f5 /proc/$$/stat)\"; \
             echo \"key $AI_CTL_API_KEY\"; echo failing >&2; exit 3",
        );
        fs::create_dir_all(root.path().join("logs")).unwrap();
        fs::write(gateway.log(), "previous run\n").unwrap();
        let listen = "127.0.0.1:8080".parse().unwrap();

        let mut started = gateway.start(&binary, "models: {}\n", &[], listen).unwrap();
        let pid = started.pid();
        assert_eq!(wait(started.as_mut()), "exit status: 3");

        let dir = root.path().join("gateway");
        let key = KeyFile::new(root.path().join("api-key"))
            .read()
            .unwrap()
            .unwrap();
        assert_eq!(key.len(), 64);
        assert_eq!(gateway.config().unwrap().as_deref(), Some("models: {}\n"));
        assert_eq!(gateway.record().unwrap(), Some(record(pid, &binary)));
        assert_eq!(
            fs::read_to_string(gateway.log()).unwrap(),
            format!(
                "{}\n-config {} -listen 127.0.0.1:8080\ngroup {pid}\nkey {}\nfailing\n",
                dir.display(),
                dir.join(CONFIG).display(),
                key
            )
        );
        assert_eq!(gateway.log_tail(2).unwrap(), format!("key {key}\nfailing"));
        assert_eq!(
            fs::read_to_string(root.path().join("logs").join(PREVIOUS_LOG)).unwrap(),
            "previous run\n"
        );
        gateway.forget().unwrap();
        assert_eq!(gateway.record().unwrap(), None);
    }

    #[test]
    fn start_writes_the_files_the_models_read_before_the_gateway_starts() {
        let root = tempfile::tempdir().unwrap();
        let gateway = gateway(root.path());
        // The gateway prints the file its model reads.
        let binary = root.path().join("llama-swap");
        let path = gateway.files_dir().join("audiocpp").join("qwen3-tts.json");
        command::script(&binary, &format!("cat {}", path.display()));
        let file = ConfigFile {
            path: path.clone(),
            content: "{\"ui\": false}\n".into(),
        };
        let listen = "127.0.0.1:8080".parse().unwrap();

        let mut started = gateway
            .start(&binary, "models: {}\n", std::slice::from_ref(&file), listen)
            .unwrap();
        assert_eq!(wait(started.as_mut()), "exit status: 0");
        assert_eq!(fs::read_to_string(gateway.log()).unwrap(), file.content);
        let mode = fs::metadata(path.parent().unwrap())
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o700);
    }

    #[test]
    fn a_file_that_cannot_be_written_starts_nothing() {
        let root = tempfile::tempdir().unwrap();
        let gateway = gateway(root.path());
        let binary = root.path().join("llama-swap");
        command::script(&binary, "touch started");
        // A directory in the file's place: the rename onto it fails.
        let path = gateway.files_dir().join("audiocpp").join("qwen3-tts.json");
        fs::create_dir_all(path.join("x")).unwrap();
        let file = ConfigFile {
            path: path.clone(),
            content: "{}\n".into(),
        };
        let listen = "127.0.0.1:8080".parse().unwrap();
        let message = format!(
            "{:#}",
            gateway
                .start(&binary, "models: {}\n", &[file], listen)
                .err()
                .unwrap()
        );
        assert!(
            message.starts_with(&format!("cannot write {}", path.display())),
            "{message}"
        );
        assert_eq!(gateway.record().unwrap(), None);
        assert!(!root.path().join("gateway").join("started").exists());
    }

    #[test]
    fn a_configuration_that_cannot_be_written_starts_nothing() {
        let root = tempfile::tempdir().unwrap();
        let gateway = gateway(root.path());
        let binary = root.path().join("llama-swap");
        command::script(&binary, "touch started");
        // A directory in the configuration's place: the rename onto it fails.
        let config = gateway.files_dir().join(CONFIG);
        fs::create_dir_all(config.join("x")).unwrap();
        let listen = "127.0.0.1:8080".parse().unwrap();
        let message = format!(
            "{:#}",
            gateway
                .start(&binary, "models: {}\n", &[], listen)
                .err()
                .unwrap()
        );
        assert!(
            message.starts_with(&format!("cannot write {}", config.display())),
            "{message}"
        );
        assert_eq!(gateway.record().unwrap(), None);
        assert!(!root.path().join("gateway").join("started").exists());
    }

    #[test]
    fn an_unusable_key_starts_nothing() {
        let root = tempfile::tempdir().unwrap();
        let gateway = gateway(root.path());
        fs::write(root.path().join("api-key"), "two words\n").unwrap();
        let binary = root.path().join("llama-swap");
        command::script(&binary, "touch started");
        let listen = "127.0.0.1:8080".parse().unwrap();
        let message = format!(
            "{:#}",
            gateway
                .start(&binary, "models: {}\n", &[], listen)
                .err()
                .unwrap()
        );
        assert!(
            message.contains("holds no API key ai-ctl can use"),
            "{message}"
        );
        assert_eq!(gateway.record().unwrap(), None);
        assert_eq!(gateway.config().unwrap(), None);
        assert!(!root.path().join("gateway").join("started").exists());
    }

    #[test]
    fn a_missing_binary_fails_to_start_and_records_nothing() {
        let root = tempfile::tempdir().unwrap();
        let gateway = gateway(root.path());
        let missing = root.path().join("llama-swap");
        let listen = "127.0.0.1:8080".parse().unwrap();
        let message = format!(
            "{:#}",
            gateway.start(&missing, "", &[], listen).err().unwrap()
        );
        assert!(message.starts_with(&format!("cannot run {}", missing.display())));
        assert_eq!(gateway.record().unwrap(), None);
    }

    #[test]
    fn a_signal_to_a_process_already_gone_is_not_an_error() {
        let gateway = gateway(Path::new("/nonexistent"));
        let mut child = command::spawn(&mut Command::new("true")).unwrap();
        let pid = child.id();
        child.wait().unwrap();
        gateway.signal(pid, Signal::Terminate).unwrap();
    }

    #[test]
    fn kill_stops_a_process() {
        let gateway = gateway(Path::new("/nonexistent"));
        let mut child = command::spawn(Command::new("sleep").arg("60")).unwrap();
        gateway.signal(child.id(), Signal::Kill).unwrap();
        assert_eq!(child.wait().unwrap().to_string(), "signal: 9 (SIGKILL)");
    }

    #[test]
    fn terminate_stops_a_process() {
        let gateway = gateway(Path::new("/nonexistent"));
        let mut child = command::spawn(Command::new("sleep").arg("60")).unwrap();
        gateway.signal(child.id(), Signal::Terminate).unwrap();
        assert_eq!(child.wait().unwrap().to_string(), "signal: 15 (SIGTERM)");
    }
}

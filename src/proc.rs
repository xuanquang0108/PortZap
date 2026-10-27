use std::process::Command;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use sysinfo::{Pid, ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize)]
pub struct ProcInfo {
    pub pid: u32,
    pub name: String,
    pub ram_bytes: u64,
}

#[derive(Debug)]
pub enum KillError {
    Denied,
    NotFound,
    Failed(String),
}

#[derive(Debug, Default, Serialize)]
pub struct KillReport {
    pub killed: Vec<(u32, String)>,
    pub denied: Vec<(u32, String)>,
    pub failed: Vec<(u32, String)>,
}

pub fn enrich(pids: &[u32]) -> Vec<ProcInfo> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let mut out: Vec<ProcInfo> = pids
        .iter()
        .filter_map(|&pid| {
            let p = sys.process(Pid::from_u32(pid))?;
            Some(ProcInfo {
                pid,
                name: p.name().to_string_lossy().into_owned(),
                ram_bytes: p.memory(),
            })
        })
        .collect();
    out.sort_by_key(|p| p.pid);
    out
}

pub fn find_by_name(query: &str) -> Vec<ProcInfo> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let query = query.to_ascii_lowercase();
    let query = query.strip_suffix(".exe").unwrap_or(&query);
    let mut out: Vec<ProcInfo> = sys
        .processes()
        .iter()
        .filter_map(|(pid, p)| {
            let name = p.name().to_string_lossy().into_owned();
            let base = name.to_ascii_lowercase();
            let base = base.strip_suffix(".exe").unwrap_or(&base);
            if base == query {
                Some(ProcInfo {
                    pid: pid.as_u32(),
                    name,
                    ram_bytes: p.memory(),
                })
            } else {
                None
            }
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name).then(a.pid.cmp(&b.pid)));
    out
}

fn is_alive(pid: u32) -> bool {
    let mut sys = System::new();
    let pids = [Pid::from_u32(pid)];
    sys.refresh_processes(ProcessesToUpdate::Some(&pids), false);
    sys.process(pids[0]).is_some()
}

fn taskkill(pid: u32, force: bool) -> Result<(), KillError> {
    let mut args: Vec<String> = vec!["/PID".into(), pid.to_string()];
    if force {
        args.push("/F".into());
    }
    args.push("/T".into());
    let out = Command::new("taskkill")
        .args(&args)
        .output()
        .map_err(|e| KillError::Failed(e.to_string()))?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let lower = text.to_ascii_lowercase();
    if out.status.success() {
        return Ok(());
    }
    if lower.contains("access is denied") || lower.contains("access denied") {
        return Err(KillError::Denied);
    }
    if lower.contains("not found") || lower.contains("no running instance") {
        return Err(KillError::NotFound);
    }
    Err(KillError::Failed(text.trim().to_string()))
}

pub fn kill_pids(pids: &[u32], force: bool) -> KillReport {
    let names: std::collections::HashMap<u32, String> = enrich(pids)
        .into_iter()
        .map(|p| (p.pid, p.name))
        .collect();
    let name_of = |pid: u32| {
        names
            .get(&pid)
            .cloned()
            .unwrap_or_else(|| format!("PID {pid}"))
    };

    let mut report = KillReport::default();
    let mut alive: Vec<u32> = Vec::new();

    if force {
        alive.extend_from_slice(pids);
    } else {
        for &pid in pids {
            match taskkill(pid, false) {
                Ok(()) => {}
                Err(KillError::Denied) => {
                    report.denied.push((pid, name_of(pid)));
                }
                Err(KillError::NotFound) => {}
                Err(KillError::Failed(e)) => {
                    report.failed.push((pid, format!("{}: {e}", name_of(pid))));
                }
            }
        }
        thread::sleep(Duration::from_millis(1500));
        for &pid in pids {
            if is_alive(pid) {
                alive.push(pid);
            }
        }
    }

    for &pid in &alive {
        match taskkill(pid, true) {
            Ok(()) => report.killed.push((pid, name_of(pid))),
            Err(KillError::Denied) => report.denied.push((pid, name_of(pid))),
            Err(KillError::NotFound) => {}
            Err(KillError::Failed(e)) => {
                report.failed.push((pid, format!("{}: {e}", name_of(pid))));
            }
        }
    }
    report
}

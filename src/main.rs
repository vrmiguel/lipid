mod ip;
mod ports;

use std::{os::unix::prelude::OsStrExt, net::IpAddr};

use fs_err as fs;
use ports::read_active_ports;
use tabled::{Tabled, Table, settings::Style};

use crate::ports::ActivePort;

pub type Result<T = ()> = anyhow::Result<T>;

fn read_pids() -> Result<impl Iterator<Item = u32>> {
    let pids = fs::read_dir("/proc/")?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .unwrap_or("0")
                .trim()
                .parse::<u32>()
                .ok()
        })
        .filter(|pid| *pid > 1);

    Ok(pids)
}

#[derive(Tabled)]
struct Entry {
    comm: String,
    pid: u32,
    address: IpAddr,
    port: u16,
    inode: u32
}

fn main() -> Result<()> {
    let mut entries = Vec::new();

    let pids = read_pids()?;
    let active_ports = read_active_ports()?;

    for pid in pids {
        let fds = format!("/proc/{pid}/fd");
        let Ok(read_dir) = fs::read_dir(fds) else {
            continue;
        };

        for maybe_entry in read_dir {
            let Ok(entry) = maybe_entry else {
                continue;
            };

            let pointed_to = fs::read_link(entry.path())?;
            let pointed_to = pointed_to.as_os_str().as_bytes();

            if let Some(remaining) = pointed_to.strip_prefix(b"socket:[") {
                debug_assert_eq!(remaining.last(), Some(&b']'));

                let inode = std::str::from_utf8(&remaining[..remaining.len() - 1])?;
                let inode: u32 = inode.parse()?;

                let relevant_port = active_ports
                    .iter()
                    .find(|active_port| active_port.inode == inode);

                match relevant_port {
                    Some(&ActivePort { address, port, inode }) => {
                        let comm = fs::read_to_string(format!("/proc/{pid}/comm"))?;
                        let comm = comm.trim_end().to_owned();
                        entries.push(Entry { comm, pid, address, port, inode });
                    },
                    None => continue,
                }
            }
        }
    }

    let mut table = Table::new(entries);
    table.with(Style::psql());
    println!("{table}");

    Ok(())
}

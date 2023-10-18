mod ip;

use std::{
    io::{self, BufRead, BufReader},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::Path,
};

use anyhow::Context;
use fs_err as fs;
use ip::parse_address_port;
use std::fs::File;

pub type Result<T = ()> = anyhow::Result<T>;

/// 0A represents LISTEN within /proc/net/tcp and /proc/net/tcp6
const LISTEN_STATUS: &str = "0A";

fn read_pids() -> Result<()> {
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

    for pid in pids {
        dbg!(pid);
    }

    Ok(())
}

#[derive(Debug)]
struct ActivePort {
    address: IpAddr,
    port: u16,
    inode: u32,
}

fn read_active_ports_ipv4() -> Result<Vec<ActivePort>> {
    let mut active_ports = Vec::new();
    let mut reader = ReallocBufReader::new("/proc/net/tcp")?;

    // Ignore the first line, contains only the headers for each column
    let _ = reader.read_line();

    while let Some(line) = reader.read_line()? {
        let mut parts = line.split_whitespace();
        let _index = parts.next();
        // local_address
        let address_and_port = parts.next();
        let _rem_address = parts.next();
        let status = parts.next();
        let inode = parts.last();

        // If the port in question is not being listened to,
        // skip to the next line
        if status != Some(LISTEN_STATUS) {
            println!("Status was {status:?}, skipping");
            continue;
        }

        let (address, port) =
            parse_address_port(address_and_port.with_context(|| "Missing address and port")?)?;

        let inode = inode.with_context(|| "Missing INODE")?.parse()?;

        active_ports.push(ActivePort { address, port, inode })
    }

    Ok(active_ports)
}

fn main() -> Result<()> {
    let active_ports = read_active_ports_ipv4()?;

    for port in active_ports {
        dbg!(port);
    }

    Ok(())
}

struct ReallocBufReader {
    reader: BufReader<File>,
    buffer: String,
}

impl ReallocBufReader {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let buffer = String::with_capacity(1024);

        Ok(Self { reader, buffer })
    }

    pub fn read_line(&mut self) -> io::Result<Option<&str>> {
        self.buffer.clear();

        let bytes_read = self.reader.read_line(&mut self.buffer)?;

        Ok((bytes_read != 0).then(|| self.buffer.as_str()))
    }
}

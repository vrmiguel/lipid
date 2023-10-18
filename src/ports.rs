use std::{
    fmt::Display,
    io::{self, BufRead, BufReader},
    net::IpAddr,
    path::Path,
};

use crate::Result;
use std::fs::File;

use crate::ip::parse_address_port;
use anyhow::Context;

/// 0A represents LISTEN within /proc/net/tcp and /proc/net/tcp6
const LISTEN_STATUS: &str = "0A";

#[derive(Debug, Clone, Copy)]
pub struct ActivePort {
    address: IpAddr,
    port: u16,
    inode: u32,
}

impl Display for ActivePort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            address,
            port,
            inode,
        } = self;
        write!(f, "{address}:{port} - {inode}")
    }
}

fn read_active_ports_from<P: AsRef<Path>>(path: P) -> Result<Vec<ActivePort>> {
    let mut active_ports = Vec::new();
    let mut reader = ReallocBufReader::new(path)?;

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
            continue;
        }

        let (address, port) =
            parse_address_port(address_and_port.with_context(|| "Missing address and port")?)?;

        let inode = inode.with_context(|| "Missing INODE")?.parse()?;

        active_ports.push(ActivePort {
            address,
            port,
            inode,
        })
    }

    Ok(active_ports)
}

pub fn read_active_ports() -> Result<Vec<ActivePort>> {
    // Get the active IPv4 addresses
    let mut active_ports = read_active_ports_from("/proc/net/tcp")?;
    // .. and then the active IPv6 ones
    let active_ports_ipv6 = read_active_ports_from("/proc/net/tcp6")?;

    // .. and then join them together
    active_ports.extend_from_slice(&active_ports_ipv6);
    Ok(active_ports)
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

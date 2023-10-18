mod ip;
mod ports;

use fs_err as fs;
use ports::read_active_ports;

pub type Result<T = ()> = anyhow::Result<T>;

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

fn main() -> Result<()> {
    let active_ports = read_active_ports()?;

    for port in active_ports {
        println!("{port} LISTEN");
    }

    Ok(())
}

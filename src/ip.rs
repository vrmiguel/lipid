use crate::Result;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use anyhow::Context;

pub fn parse_address_port(hex_pair: &str) -> Result<(IpAddr, u16)> {
    let mut parts = hex_pair.split(':');
    let hex_address = parts.next().with_context(|| "Missing IP address")?;
    let hex_port = parts.next().with_context(|| "Missing port")?;

    let ip_address = if hex_address.len() == 4 * 2 {
        // This is IPv4

        let mut buf = [0_u8; 4];
        hex::decode_to_slice(hex_address, &mut buf)?;

        Ipv4Addr::from(buf).into()
    } else if hex_address.len() == 16 * 2 {
        // This is IPv6

        let mut buf = [0_u8; 16];
        hex::decode_to_slice(hex_address, &mut buf)?;

        Ipv6Addr::from(buf).into()
    } else {
        // I don't know what this is ¯\_(ツ)_/¯
        anyhow::bail!("Uknown length for hex IP address");
    };

    let port = u16::from_str_radix(hex_port, 16)?;

    Ok((ip_address, port))
}

#[cfg(test)]
mod tests {
    use super::parse_address_port;

    #[test]
    fn parses_hex_address_port_pairs() {
        assert_eq!(
            parse_address_port("0100007F:1F90").unwrap(),
            ("1.0.0.127".parse().unwrap(), 8080)
        );

        assert_eq!(
            parse_address_port("0800A8C0:CFE6").unwrap(),
            ("8.0.168.192".parse().unwrap(), 53222)
        );
    }
}

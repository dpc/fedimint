use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use iroh::OutboundAddressPolicy;

/// Creates an Iroh outbound policy that permits only public Internet addresses.
///
/// This is a syntactic address policy. Host routing, VPNs, and destination NAT
/// can still route a permitted address to a non-public destination.
pub fn public_internet_outbound_address_policy() -> OutboundAddressPolicy {
    OutboundAddressPolicy::new(|addr| {
        if is_public_internet_address(addr.ip()) {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "non-public outbound address",
            ))
        }
    })
}

fn is_public_internet_address(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(addr) => is_public_ipv4(addr),
        IpAddr::V6(addr) => is_public_ipv6(addr),
    }
}

fn is_public_ipv4(addr: Ipv4Addr) -> bool {
    let addr = u32::from(addr);
    ![
        (0x0000_0000, 8),
        (0x0a00_0000, 8),
        (0x6440_0000, 10),
        (0x7f00_0000, 8),
        (0xa9fe_0000, 16),
        (0xac10_0000, 12),
        (0xc000_0000, 24),
        (0xc000_0200, 24),
        (0xc058_6300, 24),
        (0xc0a8_0000, 16),
        (0xc612_0000, 15),
        (0xc633_6400, 24),
        (0xcb00_7100, 24),
        (0xe000_0000, 4),
        (0xf000_0000, 4),
    ]
    .into_iter()
    .any(|(network, prefix)| ipv4_in_prefix(addr, network, prefix))
}

fn ipv4_in_prefix(addr: u32, network: u32, prefix: u32) -> bool {
    addr >> (32 - prefix) == network >> (32 - prefix)
}

fn is_public_ipv6(addr: Ipv6Addr) -> bool {
    let addr = u128::from(addr);
    if !ipv6_in_prefix(addr, 0x2000_0000_0000_0000_0000_0000_0000_0000, 3) {
        return false;
    }
    ![
        (0, 96),                                         // unspecified, loopback, IPv4-compatible
        (0x0000_0000_0000_0000_0000_ffff_0000_0000, 96), // IPv4-mapped
        (0x0064_ff9b_0000_0000_0000_0000_0000_0000, 96), // NAT64
        (0x0064_ff9b_0001_0000_0000_0000_0000_0000, 48), // local-use NAT64
        (0x0100_0000_0000_0000_0000_0000_0000_0000, 64), // discard-only
        (0x2001_0000_0000_0000_0000_0000_0000_0000, 23), // special/transition
        (0x2002_0000_0000_0000_0000_0000_0000_0000, 16), // 6to4
        (0x2001_0db8_0000_0000_0000_0000_0000_0000, 32), // documentation
        (0x3fff_0000_0000_0000_0000_0000_0000_0000, 20), // documentation
        (0x3ffe_0000_0000_0000_0000_0000_0000_0000, 16), // former 6bone
        (0xfc00_0000_0000_0000_0000_0000_0000_0000, 7),  // unique local
        (0xfe80_0000_0000_0000_0000_0000_0000_0000, 10), // link local
        (0xfec0_0000_0000_0000_0000_0000_0000_0000, 10), // site local
        (0xff00_0000_0000_0000_0000_0000_0000_0000, 8),  // multicast
    ]
    .into_iter()
    .any(|(network, prefix)| ipv6_in_prefix(addr, network, prefix))
}

fn ipv6_in_prefix(addr: u128, network: u128, prefix: u32) -> bool {
    addr >> (128 - prefix) == network >> (128 - prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denies_non_public_and_transition_addresses() {
        let policy = public_internet_outbound_address_policy();
        for address in [
            "127.0.0.1:1",
            "10.0.0.1:1",
            "169.254.1.1:1",
            "[::1]:1",
            "[fe80::1]:1",
            "[fc00::1]:1",
            "[::ffff:192.0.2.1]:1",
            "[::192.0.2.1]:1",
            "[64:ff9b::c000:201]:1",
            "[64:ff9b:1::c000:201]:1",
            "[2002:c000:0201::1]:1",
            "[2001::1]:1",
            "[2001:db8::1]:1",
            "[3fff::1]:1",
            "[100:0:0:1::1]:1",
            "[5f00::1]:1",
        ] {
            assert!(
                policy
                    .check(address.parse().expect("valid socket address"))
                    .is_err(),
                "{address}"
            );
        }
    }

    #[test]
    fn permits_public_addresses() {
        let policy = public_internet_outbound_address_policy();
        for address in ["1.1.1.1:443", "[2606:4700:4700::1111]:443"] {
            policy
                .check(address.parse().expect("valid socket address"))
                .expect("public address accepted");
        }
    }
}

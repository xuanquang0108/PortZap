use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCPROW_OWNER_PID, TCP_TABLE_OWNER_PID_LISTENER,
};

const AF_INET: u32 = 2;
const AF_INET6: u32 = 23;
const NO_ERROR: u32 = 0;

pub fn listeners_on_port(port: u16) -> Vec<u32> {
    let mut pids = Vec::new();
    for af in [AF_INET, AF_INET6] {
        for pid in listeners_table(port, af) {
            if !pids.contains(&pid) {
                pids.push(pid);
            }
        }
    }
    pids
}

fn listeners_table(port: u16, af: u32) -> Vec<u32> {
    let mut size: u32 = 0;
    unsafe {
        let _ = GetExtendedTcpTable(
            None,
            &mut size,
            true,
            af,
            TCP_TABLE_OWNER_PID_LISTENER,
            0,
        );
        if size == 0 {
            return Vec::new();
        }
        let mut buf: Vec<u8> = vec![0; size as usize];
        let rc = GetExtendedTcpTable(
            Some(buf.as_mut_ptr().cast()),
            &mut size,
            true,
            af,
            TCP_TABLE_OWNER_PID_LISTENER,
            0,
        );
        if rc != NO_ERROR {
            return Vec::new();
        }
        if af == AF_INET6 {
            parse_v6(&buf, port)
        } else {
            parse_v4(&buf, port)
        }
    }
}

fn parse_v4(buf: &[u8], port: u16) -> Vec<u32> {
    if buf.len() < 4 {
        return Vec::new();
    }
    let count = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    let rows_offset = 4usize;
    let row_size = std::mem::size_of::<MIB_TCPROW_OWNER_PID>();
    let mut pids = Vec::new();
    for i in 0..count {
        let start = rows_offset + i * row_size;
        if start + row_size > buf.len() {
            break;
        }
        let row: &MIB_TCPROW_OWNER_PID =
            unsafe { &*(buf.as_ptr().add(start).cast::<MIB_TCPROW_OWNER_PID>()) };
        let row_port = u16::from_be(row.dwLocalPort as u16);
        if row_port == port {
            pids.push(row.dwOwningPid);
        }
    }
    pids
}

fn parse_v6(buf: &[u8], port: u16) -> Vec<u32> {
    if buf.len() < 4 {
        return Vec::new();
    }
    let count = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    let rows_offset = 4usize;
    let row_size = std::mem::size_of::<MIB_TCP6ROW_OWNER_PID>();
    let mut pids = Vec::new();
    for i in 0..count {
        let start = rows_offset + i * row_size;
        if start + row_size > buf.len() {
            break;
        }
        let row: &MIB_TCP6ROW_OWNER_PID =
            unsafe { &*(buf.as_ptr().add(start).cast::<MIB_TCP6ROW_OWNER_PID>()) };
        let row_port = u16::from_be(row.dwLocalPort as u16);
        if row_port == port {
            pids.push(row.dwOwningPid);
        }
    }
    pids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_v4_finds_port() {
        let row = MIB_TCPROW_OWNER_PID {
            dwState: 2,
            dwLocalAddr: 0,
            dwLocalPort: 3000u16.swap_bytes() as u32,
            dwRemoteAddr: 0,
            dwRemotePort: 0,
            dwOwningPid: 1234,
        };
        let mut buf = Vec::new();
        buf.extend_from_slice(&1u32.to_ne_bytes());
        let bytes = unsafe {
            std::slice::from_raw_parts(
                (&row as *const MIB_TCPROW_OWNER_PID).cast::<u8>(),
                std::mem::size_of::<MIB_TCPROW_OWNER_PID>(),
            )
        };
        buf.extend_from_slice(bytes);
        assert_eq!(parse_v4(&buf, 3000), vec![1234]);
        assert!(parse_v4(&buf, 80).is_empty());
    }

    #[test]
    fn parse_v6_finds_port() {
        let row = MIB_TCP6ROW_OWNER_PID {
            ucLocalAddr: [0; 16],
            dwLocalScopeId: 0,
            dwLocalPort: 3000u16.swap_bytes() as u32,
            ucRemoteAddr: [0; 16],
            dwRemoteScopeId: 0,
            dwRemotePort: 0,
            dwState: 2,
            dwOwningPid: 4321,
        };
        let mut buf = Vec::new();
        buf.extend_from_slice(&1u32.to_ne_bytes());
        let bytes = unsafe {
            std::slice::from_raw_parts(
                (&row as *const MIB_TCP6ROW_OWNER_PID).cast::<u8>(),
                std::mem::size_of::<MIB_TCP6ROW_OWNER_PID>(),
            )
        };
        buf.extend_from_slice(bytes);
        assert_eq!(parse_v6(&buf, 3000), vec![4321]);
        assert!(parse_v6(&buf, 80).is_empty());
    }

    #[test]
    fn finds_real_listener() {
        let mut listener = None;
        for p in 45770u16..45790 {
            if let Ok(l) = std::net::TcpListener::bind(("127.0.0.1", p)) {
                listener = Some((p, l));
                break;
            }
        }
        let Some((port, _l)) = listener else {
            panic!("no free test port");
        };
        let pids = listeners_on_port(port);
        assert!(
            pids.contains(&std::process::id()),
            "expected own pid on port {port}, got {pids:?}"
        );
    }
}

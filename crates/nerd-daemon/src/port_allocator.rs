//! Dynamic internal port allocation. Ports are probed free and reserved for
//! the duration of a project run; released on stop.

use std::collections::BTreeSet;
use std::sync::Mutex;

const START_PORT: u16 = 5000;
const END_PORT: u16 = 8999;

pub struct PortAllocator {
    reserved: Mutex<BTreeSet<u16>>,
}

impl Default for PortAllocator {
    fn default() -> Self {
        Self {
            reserved: Mutex::new(BTreeSet::new()),
        }
    }
}

impl PortAllocator {
    /// Allocate a free port, preferring the requested value when free.
    pub fn allocate(&self, preferred: Option<u16>) -> Option<u16> {
        let mut reserved = self
            .reserved
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(preferred) = preferred
            && !reserved.contains(&preferred)
            && is_port_free(preferred)
        {
            reserved.insert(preferred);
            return Some(preferred);
        }
        for port in START_PORT..=END_PORT {
            if reserved.contains(&port) || !is_port_free(port) {
                continue;
            }
            reserved.insert(port);
            return Some(port);
        }
        None
    }

    pub fn release(&self, port: u16) {
        if let Ok(mut reserved) = self.reserved.lock() {
            reserved.remove(&port);
        }
    }
}

/// Best-effort TCP free check on loopback.
fn is_port_free(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

#[cfg(test)]
mod tests {
    use super::PortAllocator;

    #[test]
    fn allocates_and_releases() {
        let allocator = PortAllocator::default();
        let a = allocator.allocate(None).expect("first port");
        let b = allocator.allocate(None).expect("second port");
        assert_ne!(a, b);
        allocator.release(a);
        // A freed port can be handed out again.
        let c = allocator.allocate(Some(a)).expect("reuse port");
        assert_eq!(c, a);
    }
}

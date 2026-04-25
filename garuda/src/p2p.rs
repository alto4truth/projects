use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peer {
    pub id: String,
    pub address: String,
}

impl Peer {
    pub fn new(id: impl Into<String>, address: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            address: address.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct P2PNetwork {
    peers: HashMap<String, Peer>,
    broadcasts: Vec<String>,
    nullifiers: HashSet<String>,
}

impl P2PNetwork {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_peer(&mut self, peer: Peer) {
        self.peers.insert(peer.id.clone(), peer);
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn has_peer(&self, id: &str) -> bool {
        self.peers.contains_key(id)
    }

    pub fn broadcast(&mut self, message: impl Into<String>) {
        self.broadcasts.push(message.into());
    }

    pub fn broadcasts(&self) -> &[String] {
        &self.broadcasts
    }

    pub fn add_nullifier(&mut self, value: impl Into<String>) {
        self.nullifiers.insert(value.into());
    }

    pub fn has_nullifier(&self, value: &str) -> bool {
        self.nullifiers.contains(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_peers() {
        let mut network = P2PNetwork::new();
        network.add_peer(Peer::new("a", "127.0.0.1:9000"));
        assert_eq!(network.peer_count(), 1);
        assert!(network.has_peer("a"));
    }

    #[test]
    fn records_broadcasts_and_nullifiers() {
        let mut network = P2PNetwork::new();
        network.broadcast("hello");
        network.add_nullifier("n1");
        assert_eq!(network.broadcasts(), &["hello".to_string()]);
        assert!(network.has_nullifier("n1"));
    }
}

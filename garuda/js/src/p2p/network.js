const crypto = require('crypto');

function sha256(data) {
  return crypto.createHash('sha256').update(data).digest('hex');
}

class Peer {
  constructor(id, address, port) {
    this.id = id;
    this.address = address;
    this.port = port;
    this.connections = new Map();
    this.status = 'disconnected';
    this.lastSeen = Date.now();
    this.reputation = 100;
  }

  connect(peer) {
    this.connections.set(peer.id, peer);
    this.status = 'connected';
  }

  disconnect(peerId) {
    this.connections.delete(peerId);
  }

  updateReputation(delta) {
    this.reputation = Math.max(0, Math.min(100, this.reputation + delta));
  }
}

class P2PNetwork {
  constructor(options = {}) {
    this.peers = new Map();
    this.maxPeers = options.maxPeers || 50;
    this.messageHandlers = new Map();
    this.networkStats = { messagesSent: 0, messagesReceived: 0 };
  }

  addPeer(address, port = 8333) {
    const id = sha256(address + port);
    const peer = new Peer(id, address, port);
    this.peers.set(id, peer);
    return peer;
  }

  removePeer(peerId) {
    this.peers.delete(peerId);
  }

  broadcast(message, exclude = []) {
    this.peers.forEach((peer, id) => {
      if (!exclude.includes(id)) {
        this.networkStats.messagesSent++;
      }
    });
  }

  registerMessageHandler(type, handler) {
    this.messageHandlers.set(type, handler);
  }

  getNetworkStats() {
    return { ...this.networkStats, peerCount: this.peers.size };
  }
}

class ZKP2PNetwork extends P2PNetwork {
  constructor(options = {}) {
    super(options);
    this.zkpVerifiedPeers = new Set();
    this.nullifierSet = new Set();
    this.commitmentPool = [];
  }

  addToNullifierSet(nullifier) {
    this.nullifierSet.add(sha256(nullifier));
  }

  checkNullifier(nullifier) {
    return this.nullifierSet.has(sha256(nullifier));
  }

  getVerifiedPeers() {
    return Array.from(this.zkpVerifiedPeers);
  }
}

class DHT {
  constructor(k = 20) {
    this.k = k;
    this.table = new Map();
  }

  generateNodeId() {
    return sha256(Date.now().toString() + Math.random()).substring(0, 40);
  }

  addNode(nodeId, contact) {
    const bucketIndex = Math.floor(parseInt(nodeId.substring(0, 8), 16) / 256);
    if (!this.table.has(bucketIndex)) {
      this.table.set(bucketIndex, []);
    }
    const bucket = this.table.get(bucketIndex);
    bucket.push({ nodeId, contact });
    if (bucket.length > this.k) bucket.shift();
  }

  findClosest(targetId, count = this.k) {
    const results = [];
    this.table.forEach(bucket => {
      bucket.forEach(node => {
        results.push({ ...node, distance: Math.abs(parseInt(node.nodeId.substring(0, 8), 16) - parseInt(targetId.substring(0, 8), 16)) });
      });
    });
    return results.sort((a, b) => a.distance - b.distance).slice(0, count);
  }
}

module.exports = { Peer, P2PNetwork, ZKP2PNetwork, DHT };
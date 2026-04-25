const crypto = require('crypto');

function sha256(data) {
  return crypto.createHash('sha256').update(data).digest('hex');
}

class LRU {
  constructor(capacity) {
    this.capacity = capacity;
    this.cache = new Map();
  }

  get(key) {
    if (!this.cache.has(key)) return null;
    const value = this.cache.get(key);
    this.cache.delete(key);
    this.cache.set(key, value);
    return value;
  }

  set(key, value) {
    if (this.cache.has(key)) this.cache.delete(key);
    if (this.cache.size >= this.capacity) {
      const firstKey = this.cache.keys().next().value;
      this.cache.delete(firstKey);
    }
    this.cache.set(key, value);
  }
}

class BloomFilter {
  constructor(size = 1000, hashes = 3) {
    this.size = size;
    this.hashes = hashes;
    this.bitArray = new Uint8Array(Math.ceil(size / 8));
  }

  _hash(str, seed) {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      hash = ((hash * 31) + str.charCodeAt(i) + seed) % this.size;
    }
    return hash;
  }

  add(item) {
    for (let i = 0; i < this.hashes; i++) {
      const idx = this._hash(item, i);
      const byteIdx = Math.floor(idx / 8);
      const bitIdx = idx % 8;
      this.bitArray[byteIdx] |= (1 << bitIdx);
    }
  }

  has(item) {
    for (let i = 0; i < this.hashes; i++) {
      const idx = this._hash(item, i);
      const byteIdx = Math.floor(idx / 8);
      const bitIdx = idx % 8;
      if (!(this.bitArray[byteIdx] & (1 << bitIdx))) return false;
    }
    return true;
  }
}

class Graph {
  constructor() {
    this.adjacencyList = new Map();
  }

  addVertex(vertex) {
    if (!this.adjacencyList.has(vertex)) this.adjacencyList.set(vertex, []);
  }

  addEdge(v1, v2, weight = 1) {
    this.addVertex(v1);
    this.addVertex(v2);
    this.adjacencyList.get(v1).push({ vertex: v2, weight });
    this.adjacencyList.get(v2).push({ vertex: v1, weight });
  }

  bfs(start) {
    const visited = new Set();
    const queue = [start];
    const result = [];
    visited.add(start);
    while (queue.length) {
      const vertex = queue.shift();
      result.push(vertex);
      for (const neighbor of this.adjacencyList.get(vertex) || []) {
        if (!visited.has(neighbor.vertex)) {
          visited.add(neighbor.vertex);
          queue.push(neighbor.vertex);
        }
      }
    }
    return result;
  }

  dijkstra(start) {
    const distances = new Map();
    const visited = new Set();
    const pq = [[0, start]];
    for (const v of this.adjacencyList.keys()) distances.set(v, Infinity);
    distances.set(start, 0);
    while (pq.length) {
      pq.sort((a, b) => a[0] - b[0]);
      const [dist, vertex] = pq.shift();
      if (visited.has(vertex)) continue;
      visited.add(vertex);
      for (const neighbor of this.adjacencyList.get(vertex) || []) {
        const newDist = dist + neighbor.weight;
        if (newDist < distances.get(neighbor.vertex)) {
          distances.set(neighbor.vertex, newDist);
          pq.push([newDist, neighbor.vertex]);
        }
      }
    }
    return distances;
  }
}

class Queue {
  constructor() {
    this.items = [];
    this.waiting = [];
  }

  enqueue(item) {
    if (this.waiting.length > 0) {
      this.waiting.shift()(item);
    } else {
      this.items.push(item);
    }
  }

  async dequeue() {
    if (this.items.length > 0) return this.items.shift();
    return new Promise(resolve => this.waiting.push(resolve));
  }
}

class RateLimiter {
  constructor(maxRequests, windowMs) {
    this.maxRequests = maxRequests;
    this.windowMs = windowMs;
    this.requests = new Map();
  }

  allow(key) {
    const now = Date.now();
    const timestamps = this.requests.get(key) || [];
    const valid = timestamps.filter(t => now - t < this.windowMs);
    if (valid.length >= this.maxRequests) {
      this.requests.set(key, valid);
      return false;
    }
    valid.push(now);
    this.requests.set(key, valid);
    return true;
  }
}

class CircuitBreaker {
  constructor(threshold = 5, timeout = 60000) {
    this.threshold = threshold;
    this.timeout = timeout;
    this.failures = 0;
    this.state = 'closed';
  }

  async execute(fn) {
    if (this.state === 'open') throw new Error('Circuit breaker open');
    try {
      const result = await fn();
      this.state = 'closed';
      this.failures = 0;
      return result;
    } catch (err) {
      this.failures++;
      if (this.failures >= this.threshold) this.state = 'open';
      throw err;
    }
  }

  getState() {
    return this.state;
  }
}

class PubSub {
  constructor() {
    this.topics = new Map();
  }

  subscribe(topic, fn) {
    if (!this.topics.has(topic)) this.topics.set(topic, []);
    this.topics.get(topic).push(fn);
  }

  publish(topic, data) {
    (this.topics.get(topic) || []).forEach(fn => fn(data));
  }
}

class Cache {
  constructor(ttl = 60000) {
    this.ttl = ttl;
    this.store = new Map();
    const cleanup = setInterval(() => {
      const now = Date.now();
      for (const [key, item] of this.store) {
        if (now > item.expiry) this.store.delete(key);
      }
    }, ttl / 2);
    if (typeof cleanup.unref === 'function') cleanup.unref();
  }

  set(key, value) {
    this.store.set(key, { value, expiry: Date.now() + this.ttl });
  }

  get(key) {
    const item = this.store.get(key);
    if (!item) return null;
    if (Date.now() > item.expiry) { this.store.delete(key); return null; }
    return item.value;
  }
}

module.exports = { LRU, BloomFilter, Graph, Queue, RateLimiter, CircuitBreaker, PubSub, Cache };

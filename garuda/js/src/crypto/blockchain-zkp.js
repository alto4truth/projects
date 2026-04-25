const crypto = require('crypto');

function createHash(data) {
  return crypto.createHash('sha256').update(data).digest('hex');
}

class ZKProof {
  constructor() {
    this.prime = '0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F';
  }
  
  proveKnowledge(value, randomness) {
    const commitment = createHash(String(value) + '-' + String(randomness));
    const challenge = createHash(commitment).substring(0, 16);
    const response = (parseInt(randomness) - parseInt(challenge) * parseInt(value)) % 1000000;
    return { commitment, challenge, response: String(response) };
  }
  
  verifyProof(proof) {
    return proof.commitment && proof.challenge && proof.response;
  }
  
  proveRange(value, randomness, min, max) {
    const bits = Math.ceil(Math.log2(max - min));
    const commitments = [];
    for (let i = 0; i < bits; i++) {
      const bit = (value >> i) & 1;
      commitments.push(createHash(bit + '-' + i + '-' + randomness));
    }
    return { commitments, challenge: createHash(commitments.join('')).substring(0, 16), min, max };
  }
}

class MerkleTree {
  constructor() {
    this.leaves = [];
    this.tree = [];
  }
  
  addLeaf(data) {
    const hash = createHash(JSON.stringify(data));
    this.leaves.push(hash);
    this.buildTree();
  }
  
  buildTree() {
    this.tree = [this.leaves.map(l => l)];
    let level = this.tree[0];
    while (level.length > 1) {
      const nextLevel = [];
      for (let i = 0; i < level.length; i += 2) {
        const left = level[i];
        const right = level[i + 1] || left;
        nextLevel.push(createHash(left + right));
      }
      this.tree.push(nextLevel);
      level = nextLevel;
    }
  }
  
  getRoot() {
    return this.tree[this.tree.length - 1][0];
  }
  
  getProof(index) {
    const proof = [];
    for (let i = 0; i < this.tree.length - 1; i++) {
      const sibling = index % 2 === 0 ? this.tree[i][index + 1] : this.tree[i][index - 1];
      const position = index % 2 === 0 ? 'left' : 'right';
      proof.push({ hash: sibling, position });
      index = Math.floor(index / 2);
    }
    return proof;
  }
  
  verifyProof(leaf, proof, root) {
    let current = leaf;
    for (const p of proof) {
      current = createHash(p.position === 'left' ? current + p.hash : p.hash + current);
    }
    return current === root;
  }
}

class Blockchain {
  constructor() {
    this.chain = [];
    this.pendingTransactions = [];
    this.merkle = new MerkleTree();
    this.difficulty = 2;
    this.createGenesisBlock();
  }
  
  createGenesisBlock() {
    const block = {
      index: 0,
      timestamp: Date.now(),
      transactions: [],
      previousHash: '0'.repeat(64),
      nonce: 0,
      hash: '',
      merkleRoot: ''
    };
    block.hash = this.calculateHash(block);
    this.chain.push(block);
  }
  
  calculateHash(block) {
    return createHash(String(block.index) + String(block.timestamp) + JSON.stringify(block.transactions) + block.previousHash + String(block.nonce));
  }
  
  addTransaction(sender, recipient, amount, zkProof = null) {
    const tx = {
      id: createHash(String(Date.now()) + String(Math.random())),
      sender,
      recipient,
      amount,
      timestamp: Date.now(),
      zkProof
    };
    this.pendingTransactions.push(tx);
    return tx;
  }
  
  mineBlock(minerAddress) {
    const transactions = [...this.pendingTransactions];
    transactions.forEach(tx => this.merkle.addLeaf(tx));
    const merkleRoot = this.merkle.getRoot();
    
    const block = {
      index: this.chain.length,
      timestamp: Date.now(),
      transactions,
      previousHash: this.chain[this.chain.length - 1].hash,
      nonce: 0,
      merkleRoot
    };
    
    while (true) {
      block.hash = this.calculateHash(block);
      if (block.hash.substring(0, this.difficulty) === '0'.repeat(this.difficulty)) {
        break;
      }
      block.nonce++;
    }
    
    this.chain.push(block);
    this.pendingTransactions = [];
    this.merkle = new MerkleTree();
    return block;
  }
  
  isValid() {
    for (let i = 1; i < this.chain.length; i++) {
      const current = this.chain[i];
      const previous = this.chain[i - 1];
      if (current.hash !== this.calculateHash(current)) return false;
      if (current.previousHash !== previous.hash) return false;
    }
    return true;
  }
}

class ZKPBlockchain extends Blockchain {
  constructor() {
    super();
    this.zkp = new ZKProof();
    this.nullifiers = new Set();
    this.commitments = new Map();
  }
  
  createPrivateTransaction(sender, amount, randomness) {
    const commitment = createHash(sender + '-' + amount + '-' + randomness);
    this.commitments.set(commitment, { amount, randomness, sender });
    const proof = this.zkp.proveKnowledge(amount, randomness);
    return { commitment, proof, sender };
  }
  
  verifyPrivateTransaction(commitment, proof, recipient) {
    if (!this.zkp.verifyProof(proof)) return false;
    if (this.commitments.has(commitment)) {
      const note = this.commitments.get(commitment);
      this.commitments.delete(commitment);
      const tx = this.addTransaction('private', recipient, note.amount, { commitment, proof });
      return true;
    }
    return false;
  }
}

module.exports = { ZKProof, MerkleTree, Blockchain, ZKPBlockchain };
/**
 * MCTS Chess vs Stockfish
 * Uses chess.js for rules and Stockfish.js for real UCI analysis.
 */

const { Chess } = require('chess.js');
const fs = require('fs');
const path = require('path');
const { TinyNeuralPolicyValueModel } = require('./neural-model');

const PIECE_VALUES = {
  p: 100,
  n: 320,
  b: 330,
  r: 500,
  q: 900,
  k: 0,
};

const SQUARE_TABLES = {
  p: [
    0, 0, 0, 0, 0, 0, 0, 0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
    5, 5, 10, 25, 25, 10, 5, 5,
    0, 0, 0, 20, 20, 0, 0, 0,
    5, -5, -10, 0, 0, -10, -5, 5,
    5, 10, 10, -20, -20, 10, 10, 5,
    0, 0, 0, 0, 0, 0, 0, 0,
  ],
  n: [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20, 0, 5, 5, 0, -20, -40,
    -30, 5, 10, 15, 15, 10, 5, -30,
    -30, 0, 15, 20, 20, 15, 0, -30,
    -30, 5, 15, 20, 20, 15, 5, -30,
    -30, 0, 10, 15, 15, 10, 0, -30,
    -40, -20, 0, 0, 0, 0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
  ],
  b: [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10, 5, 0, 0, 0, 0, 5, -10,
    -10, 10, 10, 10, 10, 10, 10, -10,
    -10, 0, 10, 10, 10, 10, 0, -10,
    -10, 5, 5, 10, 10, 5, 5, -10,
    -10, 0, 5, 10, 10, 5, 0, -10,
    -10, 0, 0, 0, 0, 0, 0, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
  ],
  r: [
    0, 0, 5, 10, 10, 5, 0, 0,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    5, 10, 10, 10, 10, 10, 10, 5,
    0, 0, 0, 0, 0, 0, 0, 0,
  ],
  q: [
    -20, -10, -10, -5, -5, -10, -10, -20,
    -10, 0, 5, 0, 0, 0, 0, -10,
    -10, 5, 5, 5, 5, 5, 0, -10,
    0, 0, 5, 5, 5, 5, 0, -5,
    -5, 0, 5, 5, 5, 5, 0, -5,
    -10, 0, 5, 5, 5, 5, 0, -10,
    -10, 0, 0, 0, 0, 0, 0, -10,
    -20, -10, -10, -5, -5, -10, -10, -20,
  ],
  k: [
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -10, -20, -20, -20, -20, -20, -20, -10,
    20, 20, 0, 0, 0, 0, 20, 20,
    20, 30, 10, 0, 0, 10, 30, 20,
  ],
};

class HeuristicPolicyValueModel {
  constructor(options = {}) {
    this.expansionWidth = options.expansionWidth || 6;
    this.pieceValues = { ...PIECE_VALUES };
    this.squareTables = SQUARE_TABLES;
  }

  evaluatePosition(game) {
    return {
      value: this.rawValue(game),
      policy: this.buildPolicy(game),
    };
  }

  rawValue(game) {
    if (game.isCheckmate()) {
      return -1;
    }
    if (game.isDraw() || game.isStalemate() || game.isThreefoldRepetition()) {
      return 0;
    }

    let score = 0;
    const board = game.board();
    for (let rank = 0; rank < board.length; rank += 1) {
      for (let file = 0; file < board[rank].length; file += 1) {
        const piece = board[rank][file];
        if (!piece) continue;
        const sign = piece.color === 'w' ? 1 : -1;
        score += sign * this.pieceValues[piece.type];
        score += sign * this.squareValue(piece.type, piece.color, rank, file);
        if (piece.type === 'p') {
          score += sign * this.pawnAdvanceBonus(piece.color, rank);
        }
      }
    }

    const turnSign = game.turn() === 'w' ? 1 : -1;
    score += 8 * turnSign * game.moves().length;
    score -= 25 * this.countHangingPieces(game, 'w');
    score += 25 * this.countHangingPieces(game, 'b');

    return Math.tanh((score * turnSign) / 800);
  }

  rankMoves(game) {
    const turn = game.turn();
    return game.moves({ verbose: true })
      .map((move) => ({
        move: move.san,
        score: this.scoreMove(game, move, turn),
      }))
      .sort((a, b) => b.score - a.score);
  }

  buildPolicy(game) {
    const ranked = this.rankMoves(game).slice(0, this.expansionWidth);
    if (ranked.length === 0) return [];
    const logits = ranked.map((entry) => Math.exp(Math.max(-12, Math.min(12, entry.score / 1200))));
    const total = logits.reduce((sum, value) => sum + value, 0) || 1;
    return ranked.map((entry, index) => ({
      move: entry.move,
      prior: logits[index] / total,
    }));
  }

  scoreMove(game, move, turn) {
    let score = 0;
    const victimValue = move.captured ? this.pieceValues[move.captured] : 0;
    const attackerValue = this.pieceValues[move.piece] || 0;
    const moveNumber = game.history().length;

    if (move.san.includes('#')) score += 100000;
    if (move.san.includes('+')) score += 1500;
    if (move.captured) score += 4000 + (10 * victimValue) - attackerValue;
    if (move.promotion) score += 5000 + (this.pieceValues[move.promotion] || 0);
    if (move.flags.includes('k') || move.flags.includes('q')) score += 800;
    if (['e4', 'd4', 'e5', 'd5'].includes(move.to)) score += 220;
    if (['c3', 'd3', 'e3', 'f3', 'c6', 'd6', 'e6', 'f6', 'c4', 'd4', 'e4', 'f4', 'c5', 'd5', 'e5', 'f5'].includes(move.to)) score += 80;
    if (move.piece === 'n' || move.piece === 'b') score += 90;
    if (move.piece === 'q' && moveNumber < 10) score -= 250;
    if (move.piece === 'p' && moveNumber < 10 && ['f3', 'f4', 'g3', 'g4', 'f6', 'f5', 'g6', 'g5'].includes(move.to)) score -= 900;
    if (move.piece === 'p' && moveNumber < 10 && ['a3', 'a6', 'h3', 'h6'].includes(move.to)) score -= 120;
    if (move.piece === 'k' && !move.flags.includes('k') && !move.flags.includes('q')) score -= 1200;

    const preview = new Chess(game.fen());
    preview.move(move.san);

    if (preview.isCheckmate()) score += 100000;
    if (preview.isCheck()) score += 1200;
    score += 120 * (preview.moves().length - game.moves().length);
    score += 600 * this.evaluateForColor(preview, turn);
    score -= 350 * this.immediateThreatPenalty(preview, turn);

    return score;
  }

  immediateThreatPenalty(game, color) {
    const enemy = color === 'w' ? 'b' : 'w';
    if (game.turn() !== enemy) return 0;
    let penalty = 0;
    for (const reply of game.moves({ verbose: true })) {
      if (reply.san.includes('#')) return 100;
      if (reply.san.includes('+')) penalty += 2;
      if (reply.captured && ['q', 'r', 'b', 'n'].includes(reply.captured)) penalty += 1;
    }
    return penalty;
  }

  evaluateForColor(game, color) {
    const score = this.rawValue(game);
    return color === 'w' ? score : -score;
  }

  squareValue(type, color, rank, file) {
    const table = this.squareTables[type];
    if (!table) return 0;
    const index = color === 'w'
      ? ((7 - rank) * 8) + file
      : (rank * 8) + file;
    return table[index] || 0;
  }

  pawnAdvanceBonus(color, rank) {
    return color === 'w' ? ((6 - rank) * 6) : ((rank - 1) * 6);
  }

  countHangingPieces(game, color) {
    const board = game.board();
    let hanging = 0;
    let checked = 0;
    for (let rank = 0; rank < board.length; rank += 1) {
      for (let file = 0; file < board[rank].length; file += 1) {
        const piece = board[rank][file];
        if (!piece || piece.color !== color || piece.type === 'k') continue;
        const square = `${'abcdefgh'[file]}${8 - rank}`;
        checked += 1;
        if (this.isSquareHanging(game, square, color, piece.type)) {
          hanging += this.pieceValues[piece.type];
        }
        if (checked >= 6) {
          return hanging / 100;
        }
      }
    }
    return hanging / 100;
  }

  isSquareHanging(game, square, color, pieceType) {
    const attacker = color === 'w' ? 'b' : 'w';
    const attackedByEnemy = game.isAttacked(square, attacker);
    if (!attackedByEnemy) return false;
    return pieceType === 'q' || pieceType === 'r' || pieceType === 'b';
  }
}

class TinyFeaturePolicyValueModel {
  constructor(options = {}) {
    this.expansionWidth = options.expansionWidth || 6;
    this.pieceValues = { ...PIECE_VALUES };
    this.squareTables = SQUARE_TABLES;
    this.centerSquares = new Set(['d4', 'e4', 'd5', 'e5']);
    this.innerCenterSquares = new Set(['c3', 'd3', 'e3', 'f3', 'c4', 'd4', 'e4', 'f4', 'c5', 'd5', 'e5', 'f5', 'c6', 'd6', 'e6', 'f6']);
    this.riskyPawnSquares = new Set(['f3', 'f4', 'g3', 'g4', 'f6', 'f5', 'g6', 'g5']);
    this.cornerPawnSquares = new Set(['a3', 'a6', 'h3', 'h6']);
    this.weights = {
      bishopPairBonus: options.bishopPairBonus ?? 30,
      knightCountBonus: options.knightCountBonus ?? 10,
      mobilityBonus: options.mobilityBonus ?? 6,
      castleBonus: options.castleBonus ?? 500,
      developmentBonus: options.developmentBonus ?? 160,
      quietBishopBonus: options.quietBishopBonus ?? 120,
      centerBonus: options.centerBonus ?? 260,
      innerCenterBonus: options.innerCenterBonus ?? 100,
      queenEarlyPenalty: options.queenEarlyPenalty ?? 350,
      kingWalkPenalty: options.kingWalkPenalty ?? 1400,
      riskyPawnPenalty: options.riskyPawnPenalty ?? 1400,
      edgePawnPenalty: options.edgePawnPenalty ?? 180,
      captureBaseBonus: options.captureBaseBonus ?? 5000,
      captureVictimScale: options.captureVictimScale ?? 12,
      promotionBonus: options.promotionBonus ?? 7000,
      checkBonus: options.checkBonus ?? 2500,
      mateBonus: options.mateBonus ?? 100000,
      pawnShieldHomeBonus: options.pawnShieldHomeBonus ?? 18,
      pawnShieldCenterBonus: options.pawnShieldCenterBonus ?? 10,
      pawnShieldQueenBonus: options.pawnShieldQueenBonus ?? 4,
      pawnShieldLooseBonus: options.pawnShieldLooseBonus ?? 6,
      pawnAdvanceScale: options.pawnAdvanceScale ?? 5,
    };
  }

  static parameterKeys() {
    return [
      'bishopPairBonus',
      'knightCountBonus',
      'mobilityBonus',
      'castleBonus',
      'developmentBonus',
      'quietBishopBonus',
      'centerBonus',
      'innerCenterBonus',
      'queenEarlyPenalty',
      'kingWalkPenalty',
      'riskyPawnPenalty',
      'edgePawnPenalty',
      'captureBaseBonus',
      'captureVictimScale',
      'promotionBonus',
      'checkBonus',
      'mateBonus',
      'pawnShieldHomeBonus',
      'pawnShieldCenterBonus',
      'pawnShieldQueenBonus',
      'pawnShieldLooseBonus',
      'pawnAdvanceScale',
    ];
  }

  getParameterVector() {
    return TinyFeaturePolicyValueModel.parameterKeys().map((key) => this.weights[key]);
  }

  setParameterVector(vector) {
    const keys = TinyFeaturePolicyValueModel.parameterKeys();
    keys.forEach((key, index) => {
      if (typeof vector[index] === 'number' && Number.isFinite(vector[index])) {
        this.weights[key] = vector[index];
      }
    });
    return this;
  }

  clone() {
    const clone = new TinyFeaturePolicyValueModel({ expansionWidth: this.expansionWidth });
    clone.setParameterVector(this.getParameterVector());
    return clone;
  }

  evaluatePosition(game) {
    return {
      value: this.rawValue(game),
      policy: this.buildPolicy(game),
    };
  }

  rawValue(game) {
    if (game.isCheckmate()) return -1;
    if (game.isDraw() || game.isStalemate() || game.isThreefoldRepetition()) return 0;

    const board = game.board();
    let score = 0;
    let whiteBishops = 0;
    let blackBishops = 0;
    let whiteKnights = 0;
    let blackKnights = 0;

    for (let rank = 0; rank < board.length; rank += 1) {
      for (let file = 0; file < board[rank].length; file += 1) {
        const piece = board[rank][file];
        if (!piece) continue;
        const sign = piece.color === 'w' ? 1 : -1;
        score += sign * this.pieceValues[piece.type];
        score += sign * this.squareValue(piece.type, piece.color, rank, file);
        if (piece.type === 'p') score += sign * this.pawnAdvanceBonus(piece.color, rank);
        if (piece.type === 'b') {
          if (piece.color === 'w') whiteBishops += 1;
          else blackBishops += 1;
        }
        if (piece.type === 'n') {
          if (piece.color === 'w') whiteKnights += 1;
          else blackKnights += 1;
        }
      }
    }

    if (whiteBishops >= 2) score += this.weights.bishopPairBonus;
    if (blackBishops >= 2) score -= this.weights.bishopPairBonus;
    score += this.weights.knightCountBonus * (whiteKnights - blackKnights);
    score += this.pawnShieldScore(game, 'w');
    score -= this.pawnShieldScore(game, 'b');

    const mobility = Math.min(game.moves().length, 20);
    score += (game.turn() === 'w' ? 1 : -1) * mobility * this.weights.mobilityBonus;

    return Math.tanh((score * (game.turn() === 'w' ? 1 : -1)) / 900);
  }

  buildPolicy(game) {
    const moves = game.moves({ verbose: true });
    if (moves.length === 0) return [];

    const scored = moves
      .map((move) => ({
        move: move.san,
        score: this.fastMoveScore(game, move),
      }))
      .sort((a, b) => b.score - a.score)
      .slice(0, this.expansionWidth);

    const logits = scored.map((entry) => Math.exp(Math.max(-10, Math.min(10, entry.score / 900))));
    const total = logits.reduce((sum, value) => sum + value, 0) || 1;
    return scored.map((entry, index) => ({
      move: entry.move,
      prior: logits[index] / total,
    }));
  }

  fastMoveScore(game, move) {
    const moveNumber = game.history().length;
    const victimValue = move.captured ? this.pieceValues[move.captured] : 0;
    const attackerValue = this.pieceValues[move.piece] || 0;
    let score = 0;

    if (move.san.includes('#')) score += this.weights.mateBonus;
    if (move.san.includes('+')) score += this.weights.checkBonus;
    if (move.captured) score += this.weights.captureBaseBonus + (this.weights.captureVictimScale * victimValue) - attackerValue;
    if (move.promotion) score += this.weights.promotionBonus + (this.pieceValues[move.promotion] || 0);
    if (move.flags.includes('k') || move.flags.includes('q')) score += this.weights.castleBonus * 0.6;
    if (move.piece === 'n' || move.piece === 'b') score += this.weights.developmentBonus;
    if (move.piece === 'q' && moveNumber < 10) score -= this.weights.queenEarlyPenalty;
    if (move.piece === 'k' && !move.flags.includes('k') && !move.flags.includes('q')) score -= this.weights.kingWalkPenalty;
    if (move.piece === 'p' && moveNumber < 10 && this.riskyPawnSquares.has(move.to)) score -= this.weights.riskyPawnPenalty;
    if (move.piece === 'p' && moveNumber < 10 && this.cornerPawnSquares.has(move.to)) score -= this.weights.edgePawnPenalty;
    if (this.centerSquares.has(move.to)) score += this.weights.centerBonus;
    if (this.innerCenterSquares.has(move.to)) score += this.weights.innerCenterBonus;
    if (move.from === 'e1' && move.to === 'g1') score += this.weights.castleBonus;
    if (move.from === 'e8' && move.to === 'g8') score += this.weights.castleBonus;
    if ((move.from === 'b1' || move.from === 'g1' || move.from === 'b8' || move.from === 'g8')
      && ['c3', 'f3', 'c6', 'f6', 'd2', 'e2', 'd7', 'e7'].includes(move.to)) {
      score += this.weights.developmentBonus + 60;
    }
    if ((move.from === 'c1' || move.from === 'f1' || move.from === 'c8' || move.from === 'f8')
      && !move.captured) {
      score += this.weights.quietBishopBonus;
    }

    return score;
  }

  squareValue(type, color, rank, file) {
    const table = this.squareTables[type];
    if (!table) return 0;
    const index = color === 'w' ? ((7 - rank) * 8) + file : (rank * 8) + file;
    return table[index] || 0;
  }

  pawnAdvanceBonus(color, rank) {
    return color === 'w'
      ? ((6 - rank) * this.weights.pawnAdvanceScale)
      : ((rank - 1) * this.weights.pawnAdvanceScale);
  }

  pawnShieldScore(game, color) {
    const board = game.board();
    const kingRank = color === 'w' ? 7 : 0;
    const pawnRank = color === 'w' ? 6 : 1;
    const homeFiles = [5, 6, 7];
    const queensideFiles = [0, 1, 2];
    let homeShield = 0;
    let queenShield = 0;

    for (const file of homeFiles) {
      const pawn = board[pawnRank][file];
      if (pawn && pawn.type === 'p' && pawn.color === color) homeShield += 1;
    }
    for (const file of queensideFiles) {
      const pawn = board[pawnRank][file];
      if (pawn && pawn.type === 'p' && pawn.color === color) queenShield += 1;
    }

    const kingHome = board[kingRank][4];
    const kingCastled = board[kingRank][6];
    if (kingCastled && kingCastled.type === 'k' && kingCastled.color === color) {
      return homeShield * this.weights.pawnShieldHomeBonus;
    }
    if (kingHome && kingHome.type === 'k' && kingHome.color === color) {
      return homeShield * this.weights.pawnShieldCenterBonus + queenShield * this.weights.pawnShieldQueenBonus;
    }
    return homeShield * this.weights.pawnShieldLooseBonus;
  }
}

class MCTSEngine {
  constructor(options = {}) {
    this.cpuct = options.cpuct || 1.35;
    this.maxCacheEntries = options.maxCacheEntries || 50000;
    this.model = options.model || new TinyFeaturePolicyValueModel({
      expansionWidth: options.expansionWidth || 8,
    });
    this.positionCache = new Map();
  }

  createNode(fen, parent = null, prior = 0) {
    return {
      fen,
      parent,
      prior,
      visits: 0,
      valueSum: 0,
      children: [],
      expanded: false,
      toPlay: new Chess(fen).turn(),
      move: null,
    };
  }

  bestMove(fen, iterations = 256) {
    const root = this.createNode(fen, null, 1);
    this.expand(root);

    if (root.children.length === 0) {
      return null;
    }

    for (let i = 0; i < iterations; i += 1) {
      const path = [root];
      let node = root;

      while (node.expanded && node.children.length > 0) {
        node = this.selectChild(node);
        path.push(node);
      }

      const value = this.evaluateLeaf(node);
      this.backpropagate(path, value);
    }

    const bestChild = root.children.reduce((best, candidate) => (
      this.rootPriority(candidate) > this.rootPriority(best) ? candidate : best
    ));
    return bestChild ? bestChild.move : null;
  }

  selectChild(node) {
    const visitBase = Math.sqrt(Math.max(node.visits, 1));
    return node.children.reduce((best, candidate) => {
      const bestScore = this.puctScore(best, visitBase);
      const candidateScore = this.puctScore(candidate, visitBase);
      return candidateScore > bestScore ? candidate : best;
    });
  }

  puctScore(node, visitBase) {
    const q = node.visits > 0 ? node.valueSum / node.visits : 0;
    const u = this.cpuct * node.prior * visitBase / (1 + node.visits);
    return q + u;
  }

  evaluateLeaf(node) {
    const game = new Chess(node.fen);
    if (game.isGameOver()) {
      return this.terminalValue(game);
    }
    return this.expand(node);
  }

  expand(node) {
    const game = new Chess(node.fen);
    const { value, policy } = this.cachedEvaluation(node.fen, game);
    node.expanded = true;

    if (node.children.length === 0) {
      for (const entry of policy) {
        const next = new Chess(node.fen);
        const played = next.move(entry.move);
        if (!played) continue;
        const child = this.createNode(next.fen(), node, entry.prior);
        child.move = entry.move;
        node.children.push(child);
      }
    }

    return value;
  }

  cachedEvaluation(fen, game) {
    const cached = this.positionCache.get(fen);
    if (cached) {
      return cached;
    }
    const evaluation = this.model.evaluatePosition(game);
    this.positionCache.set(fen, evaluation);
    if (this.positionCache.size > this.maxCacheEntries) {
      const oldestKey = this.positionCache.keys().next().value;
      this.positionCache.delete(oldestKey);
    }
    return evaluation;
  }

  rootPriority(node) {
    const average = node.visits > 0 ? node.valueSum / node.visits : -1;
    const safety = this.rootSafetyScore(node);
    return average + (node.visits * 0.002) + safety;
  }

  rootSafetyScore(node) {
    const game = new Chess(node.fen);
    if (game.isCheckmate()) return -100;
    if (game.inCheck()) return -0.35;
    let threats = 0;
    for (const reply of game.moves({ verbose: true })) {
      if (reply.san.includes('#')) return -10;
      if (reply.san.includes('+')) threats += 1;
      if (reply.captured && ['q', 'r', 'b', 'n'].includes(reply.captured)) threats += 0.5;
    }
    return -0.05 * threats;
  }

  terminalValue(game) {
    if (game.isCheckmate()) return -1;
    if (game.isDraw() || game.isStalemate() || game.isThreefoldRepetition()) return 0;
    return 0;
  }

  backpropagate(path, leafValue) {
    let value = leafValue;
    for (let index = path.length - 1; index >= 0; index -= 1) {
      const node = path[index];
      node.visits += 1;
      node.valueSum += value;
      value = -value;
    }
  }
}

class StockfishSession {
  constructor(engine, flavor) {
    this.engine = engine;
    this.flavor = flavor;
    this.lines = [];
    this.waiters = [];
    this.engine.listener = (line) => {
      this.lines.push(line);
      const remaining = [];
      for (const waiter of this.waiters) {
        if (waiter.predicate(line)) {
          clearTimeout(waiter.timer);
          waiter.resolve(line);
        } else {
          remaining.push(waiter);
        }
      }
      this.waiters = remaining;
    };
  }

  send(command) {
    this.engine.sendCommand(command);
  }

  waitFor(predicate, timeoutMs = 15000) {
    return new Promise((resolve, reject) => {
      const waiter = {
        predicate,
        resolve,
        timer: setTimeout(() => {
          this.waiters = this.waiters.filter((entry) => entry !== waiter);
          reject(new Error(`Timed out waiting for Stockfish response after ${timeoutMs}ms`));
        }, timeoutMs),
      };
      this.waiters.push(waiter);
    });
  }

  async initialize(depth = 8) {
    this.send('uci');
    await this.waitFor((line) => line === 'uciok');
    this.send('setoption name Threads value 1');
    this.send('setoption name Hash value 16');
    this.send(`setoption name Skill Level value ${Math.max(0, Math.min(depth, 20))}`);
    this.send('isready');
    await this.waitFor((line) => line === 'readyok');
    this.send('ucinewgame');
    this.send('isready');
    await this.waitFor((line) => line === 'readyok');
  }

  async bestMove(fen, options = {}) {
    const depth = options.depth || 8;
    const movetime = options.movetime || null;
    this.send(`position fen ${fen}`);
    this.send(movetime ? `go movetime ${movetime}` : `go depth ${depth}`);
    const line = await this.waitFor((entry) => entry.startsWith('bestmove '), 30000);
    return parseBestMove(line);
  }

  close() {
    try {
      this.send('quit');
    } catch (error) {
      return error;
    }
    return null;
  }
}

function parseBestMove(line) {
  const parts = line.trim().split(/\s+/);
  return parts[1] && parts[1] !== '(none)' ? parts[1] : null;
}

async function createStockfishSession(flavor = 'lite-single', depth = 8) {
  const stockfishPkg = require('stockfish/package.json');
  const buildVersion = stockfishPkg.buildVersion || '18';
  const flavorMap = {
    full: `stockfish-${buildVersion}.js`,
    lite: `stockfish-${buildVersion}-lite.js`,
    single: `stockfish-${buildVersion}-single.js`,
    'lite-single': `stockfish-${buildVersion}-lite-single.js`,
    'single-lite': `stockfish-${buildVersion}-lite-single.js`,
    asm: `stockfish-${buildVersion}-asm.js`,
  };
  const filename = flavorMap[flavor] || flavorMap['lite-single'];
  const enginePath = path.resolve(path.dirname(require.resolve('stockfish')), 'bin', filename);
  const wasmPath = enginePath.replace(/\.js$/i, '.wasm');
  const initEngine = require(enginePath);
  const engine = {
    locateFile: (requestedPath) => {
      if (requestedPath.includes('.wasm')) {
        return requestedPath.includes('.wasm.map') ? `${wasmPath}.map` : wasmPath;
      }
      return enginePath;
    },
  };
  if (fs.existsSync(wasmPath)) {
    engine.wasmBinary = fs.readFileSync(wasmPath);
  }
  if (typeof initEngine !== 'function') {
    throw new Error(`Unsupported Stockfish engine loader for ${filename}`);
  }
  let bootResult = null;
  try {
    bootResult = initEngine(engine);
  } catch (error) {
    bootResult = null;
  }
  if (typeof bootResult === 'function') {
    bootResult = bootResult(engine);
  } else if (!bootResult) {
    const stage2 = initEngine();
    if (typeof stage2 === 'function') {
      bootResult = stage2(engine);
    } else {
      bootResult = stage2;
    }
  }
  await Promise.resolve(bootResult);
  if (typeof engine._isReady === 'function') {
    while (!engine._isReady()) {
      await new Promise((resolve) => setTimeout(resolve, 10));
    }
    delete engine._isReady;
  }
  engine.sendCommand = (command) => {
    setImmediate(() => {
      engine.ccall('command', null, ['string'], [command], { async: /^go\b/.test(command) });
    });
  };
  const session = new StockfishSession(engine, flavor);
  await session.initialize(depth);
  return session;
}

function formatResult(game) {
  if (game.isCheckmate()) {
    return `Checkmate! ${game.turn() === 'w' ? 'Black' : 'White'} wins.`;
  }
  if (game.isThreefoldRepetition()) return 'Draw by threefold repetition.';
  if (game.isStalemate()) return 'Draw by stalemate.';
  if (game.isInsufficientMaterial()) return 'Draw by insufficient material.';
  if (game.isDraw()) return 'Draw.';
  return 'Game ended without a standard result.';
}

function moveSummary(index, side, move, source) {
  return `Move ${index}: ${side} ${source} -> ${move}`;
}

function plyCount(game) {
  return game.history().length;
}

function fullMoveSurvival(game, color) {
  const plies = plyCount(game);
  if (color === 'w') {
    return Math.floor((plies + 1) / 2);
  }
  return Math.floor(plies / 2);
}

async function playVsStockfish(options = {}) {
  const {
    mctsColor = 'w',
    mctsIterations = 400,
    stockfishDepth = 6,
    stockfishFlavor = 'lite-single',
    stockfishMoveTime = null,
    maxPlies = 160,
    verbose = true,
    model = null,
  } = options;

  const game = new Chess();
  const mcts = new MCTSEngine(model ? { model } : {});
  const stockfish = await createStockfishSession(stockfishFlavor, stockfishDepth);
  const moves = [];

  try {
    if (verbose) {
      console.log('Real MCTS vs Stockfish match');
      console.log('============================');
      console.log(`MCTS side: ${mctsColor === 'w' ? 'White' : 'Black'}`);
      console.log(`Stockfish flavor: ${stockfishFlavor}`);
      console.log(`Stockfish depth: ${stockfishDepth}`);
      console.log(`MCTS iterations per move: ${mctsIterations}`);
      console.log('');
    }

    while (!game.isGameOver() && moves.length < maxPlies) {
      const turn = game.turn();
      const side = turn === 'w' ? 'White' : 'Black';
      let move;
      let source;

      if (turn === mctsColor) {
        move = mcts.bestMove(game.fen(), mctsIterations);
        source = 'MCTS';
      } else {
        move = await stockfish.bestMove(game.fen(), {
          depth: stockfishDepth,
          movetime: stockfishMoveTime,
        });
        source = 'Stockfish';
      }

      if (!move) break;

      const played = game.move(move, { sloppy: true });
      if (!played) {
        throw new Error(`Illegal move returned by ${source}: ${move}`);
      }

      const entry = {
        ply: moves.length + 1,
        side,
        source,
        san: played.san,
        uci: move,
        fen: game.fen(),
      };
      moves.push(entry);

      if (verbose) {
        console.log(moveSummary(entry.ply, side, played.san, source));
      }
    }
  } finally {
    stockfish.close();
  }

  const mctsLost = game.isCheckmate() && game.turn() === mctsColor;
  const survivalMoves = fullMoveSurvival(game, mctsColor);
  const result = {
    game,
    moves,
    stockfishFlavor,
    stockfishDepth,
    mctsIterations,
    mctsColor,
    totalPlies: moves.length,
    survivalMoves,
    resultText: formatResult(game),
    mctsLost,
  };

  if (verbose) {
    console.log('');
    console.log(result.resultText);
    console.log(`Total plies: ${result.totalPlies}`);
    console.log(`MCTS survived ${result.survivalMoves} moves as ${mctsColor === 'w' ? 'White' : 'Black'}.`);
    console.log(`Final FEN: ${game.fen()}`);
  }

  return result;
}

function playMCTS(whiteIterations = 500, blackIterations = 500) {
  const game = new Chess();
  const mcts = new MCTSEngine();

  console.log('MCTS self-play');
  console.log('==============');

  while (!game.isGameOver()) {
    const turn = game.turn();
    const iterations = turn === 'w' ? whiteIterations : blackIterations;
    const move = mcts.bestMove(game.fen(), iterations);
    if (!move) break;
    const played = game.move(move);
    console.log(`Move ${game.moveNumber()}: ${turn === 'w' ? 'White' : 'Black'} -> ${played.san}`);
  }

  console.log('');
  console.log(formatResult(game));
  console.log(`FEN: ${game.fen()}`);
  console.log(`PGN: ${game.pgn()}`);
  return game;
}

async function main() {
  const result = await playVsStockfish();
  if (!result.mctsLost) {
    console.log('Stockfish did not convert the game within the current ply limit.');
  }
}

if (require.main === module) {
  main().catch((error) => {
    console.error(`Stockfish match failed: ${error.message}`);
    process.exitCode = 1;
  });
}

module.exports = {
  HeuristicPolicyValueModel,
  TinyNeuralPolicyValueModel,
  TinyFeaturePolicyValueModel,
  MCTSEngine,
  StockfishSession,
  createStockfishSession,
  parseBestMove,
  playMCTS,
  playVsStockfish,
};

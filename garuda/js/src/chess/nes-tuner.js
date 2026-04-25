const { Chess } = require('chess.js');
const { MCTSEngine, TinyFeaturePolicyValueModel, TinyNeuralPolicyValueModel } = require('./mcts-stockfish');

function getModelClass(options = {}) {
  return options.modelType === 'neural'
    ? TinyNeuralPolicyValueModel
    : TinyFeaturePolicyValueModel;
}

class NESTuner {
  constructor(options = {}) {
    this.populationSize = options.populationSize || 12;
    this.sigma = options.sigma || 0.12;
    this.learningRate = options.learningRate || 0.18;
    this.seed = options.seed || 1337;
  }

  random() {
    this.seed = (1664525 * this.seed + 1013904223) >>> 0;
    return this.seed / 0x100000000;
  }

  randomNormal() {
    const u1 = Math.max(this.random(), 1e-12);
    const u2 = this.random();
    return Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
  }

  sampleNoise(length) {
    return Array.from({ length }, () => this.randomNormal());
  }

  optimize(initialVector, evaluate, iterations = 8) {
    let center = initialVector.slice();
    let bestVector = center.slice();
    let bestScore = evaluate(bestVector);
    const history = [{ iteration: 0, score: bestScore }];

    for (let iteration = 1; iteration <= iterations; iteration += 1) {
      const samples = [];
      for (let index = 0; index < this.populationSize; index += 1) {
        const noise = this.sampleNoise(center.length);
        const candidate = center.map((value, i) => value + (this.sigma * noise[i]));
        const score = evaluate(candidate);
        samples.push({ noise, score, candidate });
        if (score > bestScore) {
          bestScore = score;
          bestVector = candidate.slice();
        }
      }

      const mean = samples.reduce((sum, sample) => sum + sample.score, 0) / samples.length;
      const variance = samples.reduce((sum, sample) => sum + ((sample.score - mean) ** 2), 0) / samples.length;
      const stdev = Math.sqrt(Math.max(variance, 1e-12));
      const gradient = new Array(center.length).fill(0);

      for (const sample of samples) {
        const normalized = (sample.score - mean) / stdev;
        for (let index = 0; index < gradient.length; index += 1) {
          gradient[index] += normalized * sample.noise[index];
        }
      }

      center = center.map((value, index) => (
        value + (this.learningRate / (samples.length * this.sigma)) * gradient[index]
      ));

      const centerScore = evaluate(center);
      if (centerScore > bestScore) {
        bestScore = centerScore;
        bestVector = center.slice();
      }

      history.push({
        iteration,
        score: centerScore,
        bestScore,
      });
    }

    return {
      bestVector,
      bestScore,
      history,
    };
  }
}

function createModelFromVector(vector, options = {}) {
  const ModelClass = getModelClass(options);
  const model = new ModelClass({
    expansionWidth: options.expansionWidth || 6,
  });
  model.setParameterVector(vector);
  return model;
}

function createEngineFromVector(vector, options = {}) {
  const model = createModelFromVector(vector, options);
  return new MCTSEngine({
    model,
    cpuct: options.cpuct || 1.35,
    maxCacheEntries: options.maxCacheEntries || 50000,
  });
}

function evaluatePolicyVector(vector, options = {}) {
  const model = createModelFromVector(vector, options);
  model.setParameterVector(vector);
  const engine = createEngineFromVector(vector, options);

  const puzzles = options.positions || [
    {
      fen: new Chess().fen(),
      preferred: new Set(['d4', 'e4', 'Nf3', 'Nc3']),
      avoid: new Set(['f3', 'f4', 'g3', 'g4']),
    },
    {
      fen: 'r1bqkbnr/pppp1ppp/2n5/4p3/3P4/2N5/PPP2PPP/R1BQKBNR w KQkq - 2 3',
      preferred: new Set(['d5', 'Nf3', 'e3', 'dxe5']),
      avoid: new Set(['f4', 'g4']),
    },
    {
      fen: 'rnbqkb1r/pppp1ppp/5n2/4p3/3P4/5P2/PPP1P1PP/RNBQKBNR w KQkq - 1 3',
      preferred: new Set(['dxe5', 'e4', 'Nc3', 'Bg5']),
      avoid: new Set(['g4', 'f4']),
    },
  ];

  let score = 0;
  for (const position of puzzles) {
    const game = new Chess(position.fen);
    const modelView = model.evaluatePosition(game);
    const policyMass = new Map(modelView.policy.map((entry) => [entry.move, entry.prior]));
    const move = engine.bestMove(game.fen(), options.iterations || 20);
    if (!move) {
      score -= 5;
      continue;
    }
    score += 3 * modelView.value;
    for (const preferredMove of position.preferred) {
      score += 6 * (policyMass.get(preferredMove) || 0);
    }
    for (const avoidedMove of position.avoid) {
      score -= 8 * (policyMass.get(avoidedMove) || 0);
    }
    if (position.preferred.has(move)) score += 4;
    if (position.avoid.has(move)) score -= 6;
    if (game.moves().includes(move)) score += 1;

    const next = new Chess(game.fen());
    next.move(move);
    if (next.inCheck()) score -= 2;

    for (const reply of next.moves({ verbose: true })) {
      if (reply.san.includes('#')) score -= 12;
      if (reply.san.includes('+')) score -= 1.5;
    }
  }

  return score;
}

function scoreTerminal(game, perspectiveColor) {
  if (game.isCheckmate()) {
    return game.turn() === perspectiveColor ? -1 : 1;
  }
  return 0;
}

function playSelfPlayGame(candidateVector, opponentVector, options = {}) {
  const game = new Chess(options.startFen || new Chess().fen());
  const candidateColor = options.candidateColor || 'w';
  const candidateEngine = createEngineFromVector(candidateVector, options);
  const opponentEngine = createEngineFromVector(opponentVector, options);
  const maxPlies = options.maxPlies || 40;
  const iterations = options.iterations || 16;

  while (!game.isGameOver() && game.history().length < maxPlies) {
    const turn = game.turn();
    const engine = turn === candidateColor ? candidateEngine : opponentEngine;
    const move = engine.bestMove(game.fen(), iterations);
    if (!move) break;
    const played = game.move(move);
    if (!played) break;
  }

  const result = scoreTerminal(game, candidateColor);
  const survived = game.history().length / 2;
  const mobility = game.moves().length;

  return {
    result,
    survived,
    plies: game.history().length,
    mobility,
    pgn: game.pgn(),
    fen: game.fen(),
    game,
  };
}

function evaluateSelfPlayFitness(vector, options = {}) {
  const ModelClass = getModelClass(options);
  const baseline = options.baselineVector || new ModelClass().getParameterVector();
  const openings = options.openings || [
    'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
    'r1bqkbnr/pppp1ppp/2n5/4p3/3P4/2N5/PPP2PPP/R1BQKBNR w KQkq - 2 3',
    'rnbqkb1r/pppp1ppp/5n2/4p3/3P4/5P2/PPP1P1PP/RNBQKBNR w KQkq - 1 3',
  ];

  let score = 0;
  for (const fen of openings) {
    const asWhite = playSelfPlayGame(vector, baseline, {
      ...options,
      startFen: fen,
      candidateColor: 'w',
    });
    const asBlack = playSelfPlayGame(vector, baseline, {
      ...options,
      startFen: fen,
      candidateColor: 'b',
    });

    score += (12 * asWhite.result) + asWhite.survived + (0.05 * asWhite.mobility);
    score += (12 * asBlack.result) + asBlack.survived + (0.05 * asBlack.mobility);
  }

  return score / (openings.length * 2);
}

function evaluateMixedFitness(vector, options = {}) {
  const tactical = evaluatePolicyVector(vector, options);
  const selfPlay = evaluateSelfPlayFitness(vector, options);
  const tacticalWeight = options.tacticalWeight ?? 0.55;
  const selfPlayWeight = options.selfPlayWeight ?? 0.45;
  return (tacticalWeight * tactical) + (selfPlayWeight * selfPlay);
}

function runSmokeTune(options = {}) {
  const ModelClass = getModelClass(options);
  const model = new ModelClass();
  const initialVector = model.getParameterVector();
  const fitnessFn = options.fitnessFn || evaluateMixedFitness;
  const baselineScore = fitnessFn(initialVector, options);
  const tuner = new NESTuner(options);
  const result = tuner.optimize(
    initialVector,
    (vector) => fitnessFn(vector, options),
    options.generations || 4,
  );

  return {
    baselineScore,
    ...result,
  };
}

module.exports = {
  NESTuner,
  createEngineFromVector,
  createModelFromVector,
  evaluateMixedFitness,
  evaluatePolicyVector,
  evaluateSelfPlayFitness,
  getModelClass,
  playSelfPlayGame,
  runSmokeTune,
};

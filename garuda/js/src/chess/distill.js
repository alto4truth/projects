const { Chess } = require('chess.js');
const {
  createStockfishSession,
  playVsStockfish,
  TinyNeuralPolicyValueModel,
} = require('./mcts-stockfish');

function normalizeScore(score) {
  return Math.tanh(score / 600);
}

function parseStockfishScoreLine(line) {
  const mateMatch = line.match(/score mate (-?\d+)/);
  if (mateMatch) {
    const mate = Number(mateMatch[1]);
    return mate > 0 ? 1 : -1;
  }
  const cpMatch = line.match(/score cp (-?\d+)/);
  if (cpMatch) {
    return normalizeScore(Number(cpMatch[1]));
  }
  return 0;
}

function uciToSan(game, uci) {
  const legal = game.moves({ verbose: true });
  const match = legal.find((move) => {
    const promotion = move.promotion || '';
    return `${move.from}${move.to}${promotion}` === uci;
  });
  return match ? match.san : null;
}

async function analyzePosition(session, fen, options = {}) {
  const depth = options.depth || 8;
  const movetime = options.movetime || null;
  const before = session.lines.length;
  const bestMove = await session.bestMove(fen, { depth, movetime });
  const recent = session.lines.slice(before);
  const scoreLine = [...recent].reverse().find((line) => line.includes(' score ')) || '';
  return {
    bestMove,
    value: parseStockfishScoreLine(scoreLine),
    scoreLine,
  };
}

async function buildStockfishDataset(options = {}) {
  const openings = options.openings || [
    new Chess().fen(),
    'r1bqkbnr/pppp1ppp/2n5/4p3/3P4/2N5/PPP2PPP/R1BQKBNR w KQkq - 2 3',
    'rnbqkb1r/pppp1ppp/5n2/4p3/3P4/5P2/PPP1P1PP/RNBQKBNR w KQkq - 1 3',
  ];
  const maxPlies = options.maxPlies || 10;
  const stockfishDepth = options.stockfishDepth || 6;
  const stockfishFlavor = options.stockfishFlavor || 'lite-single';
  const session = await createStockfishSession(stockfishFlavor, stockfishDepth);
  const examples = [];

  try {
    for (const fen of openings) {
      const game = new Chess(fen);
      let plies = 0;
      while (!game.isGameOver() && plies < maxPlies) {
        const analysis = await analyzePosition(session, game.fen(), { depth: stockfishDepth });
        if (!analysis.bestMove) break;
        const bestMove = uciToSan(game, analysis.bestMove);
        if (!bestMove) break;
        examples.push({
          fen: game.fen(),
          bestMove,
          targetValue: analysis.value,
        });
        const legal = game.moves({ verbose: true });
        const teacherMove = legal.find((move) => move.san === bestMove);
        if (!teacherMove) break;
        game.move(teacherMove.san);
        plies += 1;
      }
    }
  } finally {
    session.close();
  }

  return examples;
}

async function distillAgainstStockfish(options = {}) {
  const epochs = options.epochs || 3;
  const learningRate = options.learningRate || 0.01;
  const examples = await buildStockfishDataset(options);
  const model = new TinyNeuralPolicyValueModel();
  const before = model.trainOnExamples(examples, {
    learningRate: 0,
    epochs: 1,
  });
  const result = model.trainOnExamples(examples, {
    learningRate,
    epochs,
  });
  return {
    exampleCount: examples.length,
    losses: result.losses,
    initialLoss: before.loss,
    finalLoss: result.loss,
    vector: model.getParameterVector(),
    examples,
  };
}

async function benchmarkDistilledModel(options = {}) {
  const distilled = await distillAgainstStockfish(options);
  const model = new TinyNeuralPolicyValueModel({
    parameterVector: distilled.vector,
  });
  const benchmark = await playVsStockfish({
    model,
    mctsColor: options.mctsColor || 'w',
    mctsIterations: options.mctsIterations || 64,
    stockfishDepth: options.stockfishDepth || 4,
    stockfishFlavor: options.stockfishFlavor || 'lite-single',
    stockfishMoveTime: options.stockfishMoveTime || null,
    maxPlies: options.maxPlies || 80,
    verbose: options.verbose ?? false,
  });

  return {
    ...distilled,
    benchmark: {
      totalPlies: benchmark.totalPlies,
      survivalMoves: benchmark.survivalMoves,
      resultText: benchmark.resultText,
      mctsLost: benchmark.mctsLost,
      finalFen: benchmark.game.fen(),
      pgn: benchmark.game.pgn(),
    },
  };
}

module.exports = {
  analyzePosition,
  benchmarkDistilledModel,
  buildStockfishDataset,
  distillAgainstStockfish,
  parseStockfishScoreLine,
  uciToSan,
};

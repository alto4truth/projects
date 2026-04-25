#!/usr/bin/env node

const path = require('path');
const { TinyFeaturePolicyValueModel, TinyNeuralPolicyValueModel } = require('./mcts-stockfish');
const {
  evaluateMixedFitness,
  evaluatePolicyVector,
  evaluateSelfPlayFitness,
  playSelfPlayGame,
  runSmokeTune,
} = require('./nes-tuner');
const {
  aggregateGenerationResults,
  buildGenerationManifest,
  evaluateDistributedTask,
  readJsonFile,
  runDistributedTune,
  writeJsonFile,
} = require('./distributed');
const {
  benchmarkDistilledModel,
  buildStockfishDataset,
  distillAgainstStockfish,
} = require('./distill');

function parseArgs(argv) {
  const args = { _: [] };
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token.startsWith('--')) {
      const key = token.slice(2);
      const next = argv[index + 1];
      if (next && !next.startsWith('--')) {
        args[key] = next;
        index += 1;
      } else {
        args[key] = true;
      }
    } else {
      args._.push(token);
    }
  }
  return args;
}

function parseNumber(value, fallback) {
  if (value === undefined) return fallback;
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : fallback;
}

function parseVector(input) {
  if (!input) {
    return new TinyFeaturePolicyValueModel().getParameterVector();
  }
  const parsed = JSON.parse(input);
  if (!Array.isArray(parsed)) {
    throw new Error('Expected JSON array for --vector');
  }
  return parsed.map((value) => Number(value));
}

function defaultVectorForModel(modelType) {
  return modelType === 'neural'
    ? new TinyNeuralPolicyValueModel().getParameterVector()
    : new TinyFeaturePolicyValueModel().getParameterVector();
}

function parseJson(input, label) {
  if (!input) return null;
  return JSON.parse(input);
}

function parseMaybeFile(input, filePath, label) {
  if (filePath) {
    return readJsonFile(path.resolve(filePath));
  }
  return parseJson(input, label);
}

function emitJson(payload, outputPath) {
  if (outputPath) {
    writeJsonFile(path.resolve(outputPath), payload);
  }
  console.log(JSON.stringify(payload, null, 2));
}

function selectTaskPayload(input, args) {
  if (!input) return null;
  if (input.kind === 'nes-generation-manifest') {
    const taskIndex = parseNumber(args.taskIndex, 0);
    if (!Array.isArray(input.tasks) || !input.tasks[taskIndex]) {
      throw new Error(`Manifest does not contain task index ${taskIndex}`);
    }
    return input.tasks[taskIndex];
  }
  return input;
}

function buildOptions(args) {
  return {
    iterations: parseNumber(args.iterations, 6),
    maxPlies: parseNumber(args.maxPlies, 16),
    cpuct: parseNumber(args.cpuct, 1.35),
    populationSize: parseNumber(args.populationSize, 8),
    generations: parseNumber(args.generations, 4),
    sigma: parseNumber(args.sigma, 0.12),
    learningRate: parseNumber(args.learningRate, 0.18),
    seed: parseNumber(args.seed, 1337),
    tacticalWeight: parseNumber(args.tacticalWeight, 0.55),
    selfPlayWeight: parseNumber(args.selfPlayWeight, 0.45),
    fitness: args.fitness || 'mixed',
    modelType: args.modelType || 'feature',
  };
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const command = args._[0] || 'help';
  const options = buildOptions(args);
  const vector = args.vector ? parseVector(args.vector) : defaultVectorForModel(options.modelType);

  switch (command) {
    case 'vector':
      emitJson({
        vector,
        parameterCount: vector.length,
      }, args.out);
      break;
    case 'eval':
      emitJson({
        tactical: evaluatePolicyVector(vector, options),
        selfPlay: evaluateSelfPlayFitness(vector, options),
        mixed: evaluateMixedFitness(vector, options),
      }, args.out);
      break;
    case 'selfplay': {
      const baseline = args.baseline ? parseVector(args.baseline) : defaultVectorForModel(options.modelType);
      const result = playSelfPlayGame(vector, baseline, {
        ...options,
        candidateColor: args.color || 'w',
        startFen: args.fen,
      });
      emitJson({
        result: result.result,
        survived: result.survived,
        plies: result.plies,
        mobility: result.mobility,
        fen: result.fen,
        pgn: result.pgn,
      }, args.out);
      break;
    }
    case 'tune': {
      const result = runSmokeTune({
        ...options,
        fitnessFn: args.fitness === 'tactical'
          ? evaluatePolicyVector
          : args.fitness === 'selfplay'
            ? evaluateSelfPlayFitness
            : evaluateMixedFitness,
      });
      emitJson(result, args.out);
      break;
    }
    case 'dist:plan': {
      const manifest = buildGenerationManifest({
        ...options,
        generation: parseNumber(args.generation, 1),
        centerVector: args.center ? parseVector(args.center) : vector,
      });
      emitJson(manifest, args.out);
      break;
    }
    case 'dist:worker': {
      const taskInput = parseMaybeFile(args.task, args.taskFile, 'task');
      const task = selectTaskPayload(taskInput, args);
      if (!task) {
        throw new Error('dist:worker requires --task or --taskFile');
      }
      emitJson(evaluateDistributedTask(task, options), args.out);
      break;
    }
    case 'dist:aggregate': {
      const manifest = parseMaybeFile(args.manifest, args.manifestFile, 'manifest');
      const results = parseMaybeFile(args.results, args.resultsFile, 'results');
      if (!manifest || !Array.isArray(results)) {
        throw new Error('dist:aggregate requires --manifest/--manifestFile and --results/--resultsFile');
      }
      emitJson(aggregateGenerationResults(manifest, results, options), args.out);
      break;
    }
    case 'dist:tune': {
      const result = runDistributedTune({
        ...options,
        vector,
      });
      emitJson(result, args.out);
      break;
    }
    case 'distill:data': {
      const result = await buildStockfishDataset({
        maxPlies: parseNumber(args.teacherPlies, 8),
        stockfishDepth: parseNumber(args.stockfishDepth, 6),
        stockfishFlavor: args.stockfishFlavor || 'lite-single',
      });
      emitJson({
        exampleCount: result.length,
        examples: result,
      }, args.out);
      break;
    }
    case 'distill:train': {
      const result = await distillAgainstStockfish({
        epochs: parseNumber(args.epochs, 4),
        learningRate: parseNumber(args.learningRate, 0.01),
        maxPlies: parseNumber(args.teacherPlies, 8),
        stockfishDepth: parseNumber(args.stockfishDepth, 6),
        stockfishFlavor: args.stockfishFlavor || 'lite-single',
      });
      emitJson(result, args.out);
      break;
    }
    case 'distill:benchmark': {
      const result = await benchmarkDistilledModel({
        epochs: parseNumber(args.epochs, 4),
        learningRate: parseNumber(args.learningRate, 0.01),
        teacherPlies: parseNumber(args.teacherPlies, 8),
        maxPlies: parseNumber(args.maxPlies, 80),
        stockfishDepth: parseNumber(args.stockfishDepth, 4),
        stockfishFlavor: args.stockfishFlavor || 'lite-single',
        mctsIterations: parseNumber(args.iterations, 64),
        mctsColor: args.color || 'w',
        verbose: args.verbose === 'true' || args.verbose === true,
      });
      emitJson(result, args.out);
      break;
    }
    default:
      console.log(`Usage:
  node js/src/chess/cli.js vector
  node js/src/chess/cli.js eval [--modelType feature|neural] [--vector '[...]']
  node js/src/chess/cli.js selfplay [--modelType feature|neural] [--vector '[...]'] [--baseline '[...]'] [--color w|b] [--fen FEN]
  node js/src/chess/cli.js tune [--modelType feature|neural] [--fitness mixed|selfplay|tactical] [--populationSize N] [--generations N]
  node js/src/chess/cli.js dist:plan [--fitness mixed|selfplay|tactical] [--generation N] [--center '[...]'] [--out file.json]
  node js/src/chess/cli.js dist:worker [--task '{...}' | --taskFile task.json] [--taskIndex N] [--out file.json]
  node js/src/chess/cli.js dist:aggregate [--manifest '{...}' | --manifestFile manifest.json] [--results '[...]' | --resultsFile results.json] [--out file.json]
  node js/src/chess/cli.js dist:tune [--fitness mixed|selfplay|tactical] [--populationSize N] [--generations N]
  node js/src/chess/cli.js distill:data [--stockfishDepth N] [--teacherPlies N]
  node js/src/chess/cli.js distill:train [--stockfishDepth N] [--teacherPlies N] [--epochs N] [--learningRate X]
  node js/src/chess/cli.js distill:benchmark [--stockfishDepth N] [--teacherPlies N] [--epochs N] [--learningRate X] [--iterations N] [--maxPlies N]
`);
  }
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});

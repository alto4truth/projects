const fs = require('fs');
const path = require('path');
const { TinyFeaturePolicyValueModel, TinyNeuralPolicyValueModel } = require('./mcts-stockfish');
const {
  NESTuner,
  evaluateMixedFitness,
  evaluatePolicyVector,
  evaluateSelfPlayFitness,
} = require('./nes-tuner');

function cloneVector(vector) {
  return vector.map((value) => Number(value));
}

function getFitnessFunction(name) {
  switch (name) {
    case 'tactical':
      return evaluatePolicyVector;
    case 'selfplay':
      return evaluateSelfPlayFitness;
    case 'mixed':
    default:
      return evaluateMixedFitness;
  }
}

function getInitialVector(options = {}) {
  if (Array.isArray(options.centerVector)) return cloneVector(options.centerVector);
  if (Array.isArray(options.vector)) return cloneVector(options.vector);
  return (options.modelType === 'neural'
    ? new TinyNeuralPolicyValueModel()
    : new TinyFeaturePolicyValueModel()).getParameterVector();
}

function buildGenerationManifest(options = {}) {
  const tuner = new NESTuner(options);
  const center = getInitialVector(options);
  const generation = Number.isFinite(options.generation) ? options.generation : 1;
  const fitness = options.fitness || 'mixed';
  const tasks = [];

  for (let index = 0; index < tuner.populationSize; index += 1) {
    const noise = tuner.sampleNoise(center.length);
    const vector = center.map((value, offset) => value + (tuner.sigma * noise[offset]));
    tasks.push({
      id: `g${generation}-c${index}`,
      kind: 'candidate',
      generation,
      index,
      sigma: tuner.sigma,
      fitness,
      vector,
      noise,
    });
  }

  return {
    kind: 'nes-generation-manifest',
    version: 1,
    generation,
    seedStart: options.seed || 1337,
    seedEnd: tuner.seed,
    center,
    fitness,
    populationSize: tuner.populationSize,
    sigma: tuner.sigma,
    learningRate: tuner.learningRate,
    taskCount: tasks.length,
    tasks,
  };
}

function evaluateDistributedTask(task, options = {}) {
  const fitnessName = task.fitness || options.fitness || 'mixed';
  const fitnessFn = getFitnessFunction(fitnessName);
  const score = fitnessFn(task.vector, options);
  return {
    id: task.id,
    kind: 'candidate-result',
    generation: task.generation,
    index: task.index,
    fitness: fitnessName,
    score,
    vector: cloneVector(task.vector),
  };
}

function aggregateGenerationResults(manifest, results, options = {}) {
  const resultMap = new Map();
  for (const result of results) {
    resultMap.set(result.id, result);
  }

  const samples = manifest.tasks.map((task) => {
    const result = resultMap.get(task.id);
    if (!result) {
      throw new Error(`Missing result for task ${task.id}`);
    }
    return {
      id: task.id,
      index: task.index,
      noise: task.noise,
      vector: task.vector,
      score: Number(result.score),
    };
  });

  const mean = samples.reduce((sum, sample) => sum + sample.score, 0) / Math.max(samples.length, 1);
  const variance = samples.reduce((sum, sample) => sum + ((sample.score - mean) ** 2), 0) / Math.max(samples.length, 1);
  const stdev = Math.sqrt(Math.max(variance, 1e-12));
  const gradient = new Array(manifest.center.length).fill(0);

  for (const sample of samples) {
    const normalized = (sample.score - mean) / stdev;
    for (let index = 0; index < gradient.length; index += 1) {
      gradient[index] += normalized * sample.noise[index];
    }
  }

  const nextCenter = manifest.center.map((value, index) => (
    value + (manifest.learningRate / (samples.length * manifest.sigma)) * gradient[index]
  ));

  const bestSample = samples.reduce((best, sample) => (
    !best || sample.score > best.score ? sample : best
  ), null);
  const fitnessFn = getFitnessFunction(manifest.fitness);
  const nextCenterScore = fitnessFn(nextCenter, options);

  return {
    kind: 'nes-generation-summary',
    generation: manifest.generation,
    completedTaskCount: samples.length,
    meanScore: mean,
    stdevScore: stdev,
    bestScore: bestSample ? bestSample.score : nextCenterScore,
    bestVector: bestSample ? cloneVector(bestSample.vector) : cloneVector(nextCenter),
    bestTaskId: bestSample ? bestSample.id : null,
    nextCenter,
    nextCenterScore,
    historyEntry: {
      iteration: manifest.generation,
      score: nextCenterScore,
      bestScore: Math.max(bestSample ? bestSample.score : -Infinity, nextCenterScore),
    },
  };
}

function runDistributedTune(options = {}) {
  const initialVector = getInitialVector(options);
  const fitnessName = options.fitness || 'mixed';
  const fitnessFn = getFitnessFunction(fitnessName);
  const generations = options.generations || 4;
  const baselineScore = fitnessFn(initialVector, options);
  let center = cloneVector(initialVector);
  let bestVector = cloneVector(initialVector);
  let bestScore = baselineScore;
  const history = [{ iteration: 0, score: baselineScore, bestScore }];

  for (let generation = 1; generation <= generations; generation += 1) {
    const manifest = buildGenerationManifest({
      ...options,
      generation,
      fitness: fitnessName,
      centerVector: center,
    });
    const results = manifest.tasks.map((task) => evaluateDistributedTask(task, options));
    const summary = aggregateGenerationResults(manifest, results, options);
    center = summary.nextCenter.slice();
    if (summary.bestScore > bestScore) {
      bestScore = summary.bestScore;
      bestVector = summary.bestVector.slice();
    }
    if (summary.nextCenterScore > bestScore) {
      bestScore = summary.nextCenterScore;
      bestVector = summary.nextCenter.slice();
    }
    history.push({
      iteration: generation,
      score: summary.nextCenterScore,
      bestScore,
    });
  }

  return {
    kind: 'nes-distributed-run',
    fitness: fitnessName,
    baselineScore,
    bestScore,
    bestVector,
    finalCenter: center,
    history,
  };
}

function readJsonFile(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function writeJsonFile(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

module.exports = {
  aggregateGenerationResults,
  buildGenerationManifest,
  evaluateDistributedTask,
  getFitnessFunction,
  readJsonFile,
  runDistributedTune,
  writeJsonFile,
};

const { Chess } = require('chess.js');

const INPUT_SIZE = 96;
const HIDDEN_SIZE = 32;
const MOVE_FEATURE_SIZE = 12;

function tanh(value) {
  return Math.tanh(value);
}

function softsign(value) {
  return value / (1 + Math.abs(value));
}

function squareIndex(square) {
  const file = square.charCodeAt(0) - 97;
  const rank = Number(square[1]) - 1;
  return (rank * 8) + file;
}

function encodePiece(piece) {
  switch (piece.type) {
    case 'p': return 1;
    case 'n': return 2;
    case 'b': return 3;
    case 'r': return 4;
    case 'q': return 5;
    case 'k': return 6;
    default: return 0;
  }
}

function boardTensor(game) {
  const features = new Array(INPUT_SIZE).fill(0);
  const board = game.board();
  let cursor = 0;

  for (let rank = 0; rank < board.length; rank += 1) {
    for (let file = 0; file < board[rank].length; file += 1) {
      const piece = board[rank][file];
      if (!piece) continue;
      const base = cursor % 64;
      const sign = piece.color === 'w' ? 1 : -1;
      const pieceCode = encodePiece(piece);
      features[base] += sign * (pieceCode / 6);
      features[64 + ((cursor % 16) * 2)] += sign * (piece.type === 'p' ? 0.4 : 1);
      features[64 + ((cursor % 16) * 2) + 1] += sign * ((7 - rank) / 7);
      cursor += 1;
    }
  }

  features[95] = game.turn() === 'w' ? 1 : -1;
  return features;
}

function moveFeatureVector(game, move) {
  const vector = new Array(12).fill(0);
  const moveNumber = game.history().length;
  vector[0] = squareIndex(move.from) / 63;
  vector[1] = squareIndex(move.to) / 63;
  vector[2] = encodePiece({ type: move.piece }) / 6;
  vector[3] = move.captured ? encodePiece({ type: move.captured }) / 6 : 0;
  vector[4] = move.promotion ? encodePiece({ type: move.promotion }) / 6 : 0;
  vector[5] = move.san.includes('+') ? 1 : 0;
  vector[6] = move.san.includes('#') ? 1 : 0;
  vector[7] = move.flags.includes('k') || move.flags.includes('q') ? 1 : 0;
  vector[8] = ['d4', 'e4', 'd5', 'e5'].includes(move.to) ? 1 : 0;
  vector[9] = move.piece === 'p' ? 1 : 0;
  vector[10] = Math.min(moveNumber, 20) / 20;
  vector[11] = game.turn() === 'w' ? 1 : -1;
  return vector;
}

class TinyNeuralPolicyValueModel {
  constructor(options = {}) {
    this.expansionWidth = options.expansionWidth || 8;
    this.inputSize = INPUT_SIZE;
    this.hiddenSize = HIDDEN_SIZE;
    this.moveFeatureSize = MOVE_FEATURE_SIZE;
    this.parameters = this.createDefaultParameters();
    if (Array.isArray(options.parameterVector)) {
      this.setParameterVector(options.parameterVector);
    }
  }

  createDefaultParameters() {
    const inputWeights = new Array(this.hiddenSize * this.inputSize).fill(0).map((_, index) => (
      Math.sin((index + 1) * 0.13) * 0.08
    ));
    const hiddenBias = new Array(this.hiddenSize).fill(0).map((_, index) => (
      Math.cos((index + 1) * 0.31) * 0.05
    ));
    const valueWeights = new Array(this.hiddenSize).fill(0).map((_, index) => (
      Math.sin((index + 1) * 0.41) * 0.18
    ));
    const policyInteraction = new Array(this.hiddenSize * this.moveFeatureSize).fill(0).map((_, index) => (
      Math.cos((index + 1) * 0.19) * 0.08
    ));
    const policyMoveBias = new Array(this.moveFeatureSize).fill(0).map((_, index) => (
      Math.sin((index + 1) * 0.23) * 0.06
    ));
    return {
      inputWeights,
      hiddenBias,
      valueWeights,
      valueBias: 0,
      policyInteraction,
      policyMoveBias,
      moveBias: 0,
    };
  }

  static parameterKeys() {
    return [
      'inputWeights',
      'hiddenBias',
      'valueWeights',
      'valueBias',
      'policyInteraction',
      'policyMoveBias',
      'moveBias',
    ];
  }

  getParameterVector() {
    return [
      ...this.parameters.inputWeights,
      ...this.parameters.hiddenBias,
      ...this.parameters.valueWeights,
      this.parameters.valueBias,
      ...this.parameters.policyInteraction,
      ...this.parameters.policyMoveBias,
      this.parameters.moveBias,
    ];
  }

  setParameterVector(vector) {
    let offset = 0;
    const assignSlice = (target, length) => {
      for (let index = 0; index < length; index += 1) {
        const next = vector[offset + index];
        if (typeof next === 'number' && Number.isFinite(next)) {
          target[index] = next;
        }
      }
      offset += length;
    };

    assignSlice(this.parameters.inputWeights, this.hiddenSize * this.inputSize);
    assignSlice(this.parameters.hiddenBias, this.hiddenSize);
    assignSlice(this.parameters.valueWeights, this.hiddenSize);
    if (typeof vector[offset] === 'number' && Number.isFinite(vector[offset])) {
      this.parameters.valueBias = vector[offset];
    }
    offset += 1;
    assignSlice(this.parameters.policyInteraction, this.hiddenSize * this.moveFeatureSize);
    assignSlice(this.parameters.policyMoveBias, this.moveFeatureSize);
    if (typeof vector[offset] === 'number' && Number.isFinite(vector[offset])) {
      this.parameters.moveBias = vector[offset];
    }
    return this;
  }

  clone() {
    return new TinyNeuralPolicyValueModel({
      expansionWidth: this.expansionWidth,
      parameterVector: this.getParameterVector(),
    });
  }

  hiddenState(input) {
    const output = new Array(this.hiddenSize).fill(0);
    const preactivations = new Array(this.hiddenSize).fill(0);
    for (let hidden = 0; hidden < this.hiddenSize; hidden += 1) {
      let sum = this.parameters.hiddenBias[hidden];
      const base = hidden * this.inputSize;
      for (let index = 0; index < this.inputSize; index += 1) {
        sum += this.parameters.inputWeights[base + index] * input[index];
      }
      preactivations[hidden] = sum;
      output[hidden] = softsign(sum);
    }
    return { hidden: output, preactivations };
  }

  policyContext(hidden) {
    const context = this.parameters.policyMoveBias.slice();
    for (let hiddenIndex = 0; hiddenIndex < this.hiddenSize; hiddenIndex += 1) {
      const value = hidden[hiddenIndex];
      const base = hiddenIndex * this.moveFeatureSize;
      for (let featureIndex = 0; featureIndex < this.moveFeatureSize; featureIndex += 1) {
        context[featureIndex] += value * this.parameters.policyInteraction[base + featureIndex];
      }
    }
    return context;
  }

  forward(game) {
    const input = boardTensor(game);
    const { hidden, preactivations } = this.hiddenState(input);
    const value = tanh(
      hidden.reduce((sum, node, index) => sum + (node * this.parameters.valueWeights[index]), this.parameters.valueBias)
    );
    const context = this.policyContext(hidden);

    const moves = game.moves({ verbose: true });
    if (moves.length === 0) {
      return {
        value,
        policy: [],
        input,
        hidden,
        preactivations,
        context,
        moves: [],
        logits: [],
      };
    }

    const ranked = moves.map((move) => {
      const moveFeatures = moveFeatureVector(game, move);
      let logit = this.parameters.moveBias;
      for (let index = 0; index < moveFeatures.length; index += 1) {
        logit += moveFeatures[index] * context[index];
      }
      return {
        move: move.san,
        prior: logit,
        features: moveFeatures,
      };
    }).sort((a, b) => b.prior - a.prior);

    const shortlisted = ranked.slice(0, this.expansionWidth);

    const logits = shortlisted.map((entry) => Math.exp(Math.max(-10, Math.min(10, entry.prior))));
    const total = logits.reduce((sum, entry) => sum + entry, 0) || 1;
    const policy = shortlisted.map((entry, index) => ({
      move: entry.move,
      prior: logits[index] / total,
    }));

    return {
      value,
      policy,
      input,
      hidden,
      preactivations,
      context,
      moves: shortlisted,
      logits,
    };
  }

  evaluatePosition(game) {
    const forward = this.forward(game);
    return {
      value: forward.value,
      policy: forward.policy,
    };
  }

  trainOnExamples(examples, options = {}) {
    const learningRate = options.learningRate ?? 0.01;
    const epochs = options.epochs ?? 1;
    const policyWeight = options.policyWeight ?? 1.0;
    const valueWeight = options.valueWeight ?? 0.25;
    const losses = [];

    for (let epoch = 0; epoch < epochs; epoch += 1) {
      let epochLoss = 0;
      for (const example of examples) {
        const game = new Chess(example.fen);
        const forward = this.forward(game);
        if (forward.moves.length === 0) continue;

        const targetIndex = forward.moves.findIndex((move) => move.move === example.bestMove);
        if (targetIndex === -1) continue;

        const probabilities = forward.policy.map((entry) => entry.prior);
        const targetValue = typeof example.targetValue === 'number' ? example.targetValue : 0;
        const valueError = forward.value - targetValue;
        const clippedProb = Math.max(probabilities[targetIndex], 1e-9);
        const policyLoss = -Math.log(clippedProb);
        const valueLoss = valueError * valueError;
        epochLoss += (policyWeight * policyLoss) + (valueWeight * valueLoss);

        const gradMoveBias = 0;
        const gradPolicyMoveBias = new Array(this.moveFeatureSize).fill(0);
        const gradPolicyInteraction = new Array(this.hiddenSize * this.moveFeatureSize).fill(0);
        const gradValueWeights = new Array(this.hiddenSize).fill(0);
        let gradValueBias = 0;
        const gradHidden = new Array(this.hiddenSize).fill(0);

        const dValuePre = 2 * valueWeight * valueError * (1 - (forward.value * forward.value));
        for (let index = 0; index < this.hiddenSize; index += 1) {
          gradValueWeights[index] += dValuePre * forward.hidden[index];
          gradHidden[index] += dValuePre * this.parameters.valueWeights[index];
        }
        gradValueBias += dValuePre;

        const accumulatedMoveFeatureGrad = new Array(this.moveFeatureSize).fill(0);
        for (let moveIndex = 0; moveIndex < forward.moves.length; moveIndex += 1) {
          const expected = moveIndex === targetIndex ? 1 : 0;
          const dLogit = policyWeight * (probabilities[moveIndex] - expected);
          const move = forward.moves[moveIndex];
          for (let featureIndex = 0; featureIndex < this.moveFeatureSize; featureIndex += 1) {
            const contribution = dLogit * move.features[featureIndex];
            accumulatedMoveFeatureGrad[featureIndex] += contribution;
            gradPolicyMoveBias[featureIndex] += contribution;
          }
        }

        for (let hiddenIndex = 0; hiddenIndex < this.hiddenSize; hiddenIndex += 1) {
          const base = hiddenIndex * this.moveFeatureSize;
          for (let featureIndex = 0; featureIndex < this.moveFeatureSize; featureIndex += 1) {
            const interactionGrad = forward.hidden[hiddenIndex] * accumulatedMoveFeatureGrad[featureIndex];
            gradPolicyInteraction[base + featureIndex] += interactionGrad;
            gradHidden[hiddenIndex] += this.parameters.policyInteraction[base + featureIndex] * accumulatedMoveFeatureGrad[featureIndex];
          }
        }

        for (let hiddenIndex = 0; hiddenIndex < this.hiddenSize; hiddenIndex += 1) {
          const preactivation = forward.preactivations[hiddenIndex];
          const derivative = 1 / ((1 + Math.abs(preactivation)) ** 2);
          const dPreactivation = gradHidden[hiddenIndex] * derivative;
          this.parameters.hiddenBias[hiddenIndex] -= learningRate * dPreactivation;
          const base = hiddenIndex * this.inputSize;
          for (let inputIndex = 0; inputIndex < this.inputSize; inputIndex += 1) {
            this.parameters.inputWeights[base + inputIndex] -= learningRate * dPreactivation * forward.input[inputIndex];
          }
        }

        for (let index = 0; index < this.hiddenSize; index += 1) {
          this.parameters.valueWeights[index] -= learningRate * gradValueWeights[index];
        }
        this.parameters.valueBias -= learningRate * gradValueBias;
        for (let index = 0; index < gradPolicyMoveBias.length; index += 1) {
          this.parameters.policyMoveBias[index] -= learningRate * gradPolicyMoveBias[index];
        }
        for (let index = 0; index < gradPolicyInteraction.length; index += 1) {
          this.parameters.policyInteraction[index] -= learningRate * gradPolicyInteraction[index];
        }
        this.parameters.moveBias -= learningRate * gradMoveBias;
      }
      losses.push(examples.length > 0 ? epochLoss / examples.length : 0);
    }

    return {
      loss: losses[losses.length - 1] ?? 0,
      losses,
    };
  }
}

module.exports = {
  TinyNeuralPolicyValueModel,
  boardTensor,
  moveFeatureVector,
};

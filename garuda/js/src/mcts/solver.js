const { Node, Tree } = require('./tree');

class MCTSTokenSolver {
  constructor() {
    this.explorationConstant = 1.414;
    this.maxIterations = 512;
    this.maxDepth = 24;
  }

  solve(problem) {
    const tree = this.createRootNode(problem);
    tree.root.setUntriedActions(problem.operations);

    for (let i = 0; i < this.maxIterations; i++) {
      const node = this.select(tree);
      const expanded = this.expand(node, problem);
      const result = this.simulate(expanded, problem);

      if (result === problem.target) {
        return this.backtrace(expanded);
      }

      this.backpropagate(expanded, result === problem.target ? 1 : 0);
    }

    return this.bestSolution(tree);
  }

  createRootNode(problem) {
    const initialState = {
      tokens: [{ type: 'number', value: problem.start.toString() }],
      expression: problem.start.toString(),
    };
    return new Tree(new Node(initialState, null));
  }

  select(tree) {
    let node = tree.root;

    while (!node.isLeaf() && node.isFullyExpanded()) {
      node = this.bestChild(node);
      if (!node) break;
    }

    return node;
  }

  expand(node, problem) {
    if (node.isLeaf()) {
      return this.expandNode(node, problem);
    }

    const unvisited = node.getUntriedActions(problem);
    if (unvisited.length > 0) {
      const action = unvisited[Math.floor(Math.random() * unvisited.length)];
      return this.applyAction(node, action, problem);
    }

    return node;
  }

  expandNode(node, problem) {
    const actions = node.getUntriedActions(problem);
    if (actions.length === 0) return node;

    const action = actions[Math.floor(Math.random() * actions.length)];
    return this.applyAction(node, action, problem);
  }

  applyAction(node, action, problem) {
    const newTokens = [...node.state.tokens];
    newTokens.push({ type: 'operator', value: action, precedence: 2 });

    const currentValue = this.evaluate(node.state.expression);
    const operand = this.chooseOperand(currentValue, action, problem);
    const result = this.applyOperation(currentValue, action, operand);

    newTokens.push({ type: 'number', value: operand.toString() });

    const newExpression = newTokens.map(t => t.value).join(' ');

    const newState = { tokens: newTokens, expression: newExpression };
    return node.addChild(newState);
  }

  chooseOperand(currentValue, action, problem) {
    const target = typeof problem.target === 'number' ? problem.target : currentValue;
    const distance = target - currentValue;

    switch (action) {
      case '+':
        return distance !== 0 ? distance : Math.max(1, Math.abs(currentValue));
      case '-':
        return distance !== 0 ? -distance : 1;
      case '*':
        if (currentValue !== 0 && target % currentValue === 0) {
          return target / currentValue;
        }
        return target >= currentValue ? 2 : 0.5;
      case '/':
        if (target !== 0 && currentValue % target === 0) {
          return currentValue / target;
        }
        return 2;
      case '**':
        return 2;
      case 'sqrt':
        return 0;
      case 'sin':
      case 'cos':
        return 0;
      default:
        return distance !== 0 ? distance : 1;
    }
  }

  simulate(node, problem) {
    let currentNode = node;
    let depth = 0;

    while (depth < this.maxDepth) {
      const value = this.evaluate(currentNode.state.expression);

      if (value === problem.target) {
        return problem.target;
      }

      if (Math.abs(value - problem.target) < 0.0001) {
        return problem.target;
      }

      const actions = problem.operations;
      if (actions.length === 0) break;

      const randomAction = actions[Math.floor(Math.random() * actions.length)];
      currentNode = this.applyAction(currentNode, randomAction, problem);
      depth++;
    }

    return this.evaluate(currentNode.state.expression);
  }

  bestChild(node) {
    if (node.children.length === 0) return null;

    let bestScore = -Infinity;
    let bestNode = null;

    for (const child of node.children) {
      const score = this.uct(child);
      if (score > bestScore) {
        bestScore = score;
        bestNode = child;
      }
    }

    return bestNode;
  }

  uct(node) {
    const parentVisits = node.parent ? node.parent.visits : 1;
    const nodeVisits = node.visits || 1;
    return node.score / nodeVisits + this.explorationConstant * Math.sqrt(Math.log(parentVisits) / nodeVisits);
  }

  backpropagate(node, reward) {
    let current = node;
    while (current) {
      current.visits++;
      current.score += reward;
      current = current.parent;
    }
  }

  backtrace(node) {
    const path = [];
    let current = node;
    while (current) {
      path.unshift(current);
      current = current.parent;
    }

    return path.length > 0 ? path[path.length - 1].state.tokens : [];
  }

  bestSolution(tree) {
    let bestScore = -Infinity;
    let bestNode = null;

    const candidates = tree.root.children;
    for (const node of candidates) {
      if (node.visits > 0 && node.score / node.visits > bestScore) {
        bestScore = node.score / node.visits;
        bestNode = node;
      }
    }

    return bestNode ? bestNode.state.tokens : null;
  }

  evaluate(expression) {
    try {
      const tokens = expression.split(' ').filter(t => t);
      if (tokens.length < 2) return parseFloat(tokens[0] || '0');

      let result = parseFloat(tokens[0]);

      for (let i = 1; i < tokens.length; i += 2) {
        const op = tokens[i];
        const num = parseFloat(tokens[i + 1]);
        result = this.applyOperation(result, op, num);
      }

      return result;
    } catch {
      return 0;
    }
  }

  applyOperation(a, op, b) {
    const operand = b !== undefined ? b : 0;
    switch (op) {
      case '+': return a + operand;
      case '-': return a - operand;
      case '*': return a * operand;
      case '/': return operand !== 0 ? a / operand : 0;
      case '**': return Math.pow(a, operand);
      case 'sqrt': return Math.sqrt(a);
      case 'sin': return Math.sin(a);
      case 'cos': return Math.cos(a);
      default: return a;
    }
  }
}

module.exports = { MCTSTokenSolver };

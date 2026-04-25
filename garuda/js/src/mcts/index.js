/**
 * MCTS (Monte Carlo Tree Search) Library
 * 
 * @module mcts
 * @description Token-level MCTS solver for mathematical problems
 */

const { Node, Tree } = require('./tree');
const { MCTSTokenSolver } = require('./solver');

/**
 * Create a new MCTS solver
 * @function createSolver
 * @param {Object} options - Solver options
 * @param {number} options.explorationConstant - UCB exploration constant (default: 1.414)
 * @param {number} options.maxIterations - Max iterations (default: 10000)
 * @param {number} options.maxDepth - Max tree depth (default: 50)
 * @returns {MCTSTokenSolver}
 */
function createSolver(options = {}) {
  const solver = new MCTSTokenSolver();
  if (options.explorationConstant) solver.explorationConstant = options.explorationConstant;
  if (options.maxIterations) solver.maxIterations = options.maxIterations;
  if (options.maxDepth) solver.maxDepth = options.maxDepth;
  return solver;
}

/**
 * Solve a mathematical problem
 * @function solve
 * @param {Object} problem
 * @param {number} problem.target - Target value
 * @param {number} problem.start - Starting value  
 * @param {string[]} problem.operations - Available operations
 * @returns {Array} Solution tokens
 */
function solve(problem) {
  const solver = createSolver();
  return solver.solve(problem);
}

/**
 * Create a new tree node
 * @function createNode
 * @param {any} state - Node state
 * @param {Node} parent - Parent node
 * @returns {Node}
 */
function createNode(state, parent = null) {
  return new Node(state, parent);
}

/**
 * Create a new tree
 * @function createTree
 * @param {Node} root - Root node
 * @returns {Tree}
 */
function createTree(root) {
  return new Tree(root);
}

/**
 * MCTS Library version
 */
const VERSION = '1.0.0';

module.exports = {
  VERSION,
  Node,
  Tree,
  MCTSTokenSolver,
  createSolver,
  solve,
  createNode,
  createTree,
};
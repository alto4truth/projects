class Node {
  constructor(state, parent = null) {
    this.state = state;
    this.parent = parent;
    this.children = [];
    this.visits = 0;
    this.score = 0;
    this._untriedActions = [];
  }

  isLeaf() {
    return this.children.length === 0;
  }

  isFullyExpanded() {
    return this._untriedActions.length === 0;
  }

  addChild(state) {
    const child = new Node(state, this);
    this.children.push(child);
    return child;
  }

  getUntriedActions(problem) {
    return this._untriedActions;
  }

  setUntriedActions(actions) {
    this._untriedActions = actions;
  }

  isTerminal() {
    return false;
  }
}

class Tree {
  constructor(root) {
    this.root = root;
  }
}

module.exports = { Node, Tree };
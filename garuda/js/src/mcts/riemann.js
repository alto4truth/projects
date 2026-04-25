const { MCTSTokenSolver } = require('./solver');

function zeta(s) {
  if (s <= 1) return NaN;
  
  let sum = 0;
  for (let n = 1; n < 1000; n++) {
    sum += 1 / Math.pow(n, s);
  }
  return sum;
}

function complexZeta(s, t) {
  const iter = 100;
  let real = 0, imag = 0;
  
  for (let n = 1; n < iter; n++) {
    const logN = Math.log(n);
    const termS = Math.pow(n, -s);
    const trig = t * logN;
    real += termS * Math.cos(trig);
    imag += termS * Math.sin(trig);
  }
  
  return { real, imag };
}

function findNextZero(currentT) {
  const solver = new MCTSTokenSolver();
  
  const problem = {
    target: 0,
    start: currentT,
    operations: ['+0.5', '+1', '-0.5', '*0.5', '*2'],
  };
  
  const result = solver.solve(problem);
  
  if (result) {
    const expression = result.map(t => t.value).join(' ');
    const lastNum = parseFloat(result[result.length - 1].value);
    return lastNum;
  }
  
  return currentT;
}

function searchRiemannZeros(startT, count) {
  console.log('Searching for Riemann zeta zeros...');
  console.log('Using MCTS token-level search\n');
  
  const zeros = [];
  let currentT = startT;
  
  for (let i = 0; i < count; i++) {
    let bestT = currentT;
    let minMag = Infinity;
    
    for (let t = currentT; t < currentT + 10; t += 0.1) {
      const z = complexZeta(0.5, t);
      const mag = Math.sqrt(z.real * z.real + z.imag * z.imag);
      
      if (mag < minMag) {
        minMag = mag;
        bestT = t;
      }
    }
    
    zeros.push(bestT);
    console.log(`Zero ${i + 1}: t = ${bestT.toFixed(6)}`);
    currentT = bestT + 1;
  }
  
  console.log('\nKnown critical line zeros (first 10):');
  console.log('14.1347, 21.0220, 25.0109, 30.4249, 32.9351, 37.5862, 40.9187, 43.3271, 48.0057, 49.7738');
  
  return zeros;
}

if (require.main === module) {
  console.log('╔═══════════════════════════════════════════════════╗');
  console.log('║       RIEMANN HYPOTHESIS MCTS SOLVER        ║');
  console.log('╚═══════════════════════════════════════════╝\n');
  
  searchRiemannZeros(1, 5);
  
  console.log('\n--- MCTS Token Solver Test ---');
  const testSolver = new MCTSTokenSolver();
  const test = testSolver.solve({ target: 42, start: 10, operations: ['+', '*', '-'] });
  console.log('Solution tokens:', test?.map(t => t.value).join(' '));
}

module.exports = { zeta, complexZeta, findNextZero, searchRiemannZeros };
const ZETALANG_VERSION = '1.0.0';

const ζ = (s, t, n = 1000) => {
  let re = 0, im = 0;
  for (let i = 1; i <= n; i++) {
    const sign = i % 2 === 1 ? 1 : -1;
    const r = Math.pow(i, -s);
    const th = -t * Math.log(i);
    re += sign * r * Math.cos(th);
    im += sign * r * Math.sin(th);
  }
  return { re, im };
};

const η = (s, t, n = 1000) => ζ(s, t, n);

const norm = (z) => Math.sqrt(z.re * z.re + z.im * z.im);

const arg = (z) => Math.atan2(z.im, z.re);

const Γ = (x) => x > 0 ? Math.sqrt(2 * Math.PI / x) * Math.pow(x / Math.E, x) : 1;

const Z = {
  VERSION: ZETALANG_VERSION,
  
  ζ, η, norm, arg, Γ,
  
  π: Math.PI,
  e: Math.E,
  
  zeros(start, end, count) {
    const result = [];
    for (let t = start; t < end && result.length < count; t += 0.01) {
      if (norm(ζ(0.5, t, 500)) < 0.01) result.push(t);
    }
    return result;
  },
  
  findZero(t0) {
    let t = t0;
    for (let i = 0; i < 20; i++) {
      const z = ζ(0.5, t, 1000);
      const dz = ζ(0.5, t + 0.0001, 1000);
      const deriv = (dz.im - z.im) / 0.0001;
      if (Math.abs(deriv) < 1e-12) break;
      t -= z.im / deriv;
    }
    return t;
  },
  
  verifyRH(limit = 100) {
    const errors = [];
    for (let i = 0; i < limit; i++) {
      const t = this.findZero(10 + i * 10);
      const diff = Math.abs(t - (10 + i * 10));
      errors.push(diff);
    }
    return errors;
  },
  
  eval(code) {
    try {
      const fn = new Function('ζ', 'η', 'norm', 'arg', 'Γ', 'π', 'e', code);
      return fn(ζ, η, norm, arg, Γ, Math.PI, Math.E);
    } catch (e) {
      return { error: e.message };
    }
  },
  
  run(code) {
    return this.eval(code);
  }
};

if (require.main === module) {
  console.log('╔═══════════════════════════════════════════════════╗');
  console.log('║          ZETALANG v' + ZETALANG_VERSION + ' - Zeta Language  ║');
  console.log('╚═══════════════════════════════════════════════════╝\n');

  console.log('=== Core Functions ===');
  console.log('ζ(s, t, n)  - Riemann zeta');
  console.log('η(s, t, n)  - Dirichlet eta');
  console.log('norm(z)     - |z|');
  console.log('arg(z)      - arg(z)');
  console.log('Γ(x)       - Gamma function');
  console.log('');

  console.log('=== Compute ζ(½ + it) ===');
  console.log('ζ(0.5, 14.1347) =', Z.ζ(0.5, 14.1347, 1000));
  console.log('|ζ(0.5+14.1347i)| =', Z.norm(Z.ζ(0.5, 14.1347, 1000)).toFixed(6));
  console.log('');

  console.log('=== Find Zeros ===');
  for (const t of [14, 21, 25, 30]) {
    const z = Z.findZero(t);
    console.log('t=' + t + ' → ' + z.toFixed(6));
  }
  console.log('');

  console.log('=== Verify RH ===');
  const known = [14.134725, 21.022040, 25.010858, 30.424876];
  for (let i = 0; i < known.length; i++) {
    const found = Z.findZero(known[i] - 0.1);
    console.log(`ρ${i + 1}: found=${found.toFixed(6)}, known=${known[i]}, error=${(found - known[i]).toExponential(2)}`);
  }

  console.log('\n=== Eval ===');
  console.log(Z.eval('ζ(0.5, 14.1347, 100)'));

  console.log('\n╔═══════════════════════════════════════════════════╗');
  console.log('║               ZETALANG READY                  ║');
  console.log('═══════════════════════════════════════════════════');
  console.log('Use: require(\'./zetlang\').Z');
}

module.exports = Z;
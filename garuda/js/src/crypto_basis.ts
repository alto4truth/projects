import * as crypto from "crypto";

export type TestResult = [string, boolean, string, unknown, unknown];

export interface CryptoBasis {
  sha256(data: Buffer): Buffer;
  sha512(data: Buffer): Buffer;
  hmacSha256(key: Buffer, data: Buffer): Buffer;
  hmacSha512(key: Buffer, data: Buffer): Buffer;
  randomBytes(n: number): Buffer;
  md5(data: Buffer): Buffer;
  sha1(data: Buffer): Buffer;
  sha3_256(data: Buffer): Buffer;
  sha3_512(data: Buffer): Buffer;
  blake2b(data: Buffer, digestSize?: number): Buffer;
  blake2s(data: Buffer, digestSize?: number): Buffer;
}

export const CryptoBasis = {
  sha256(data: Buffer): Buffer {
    return crypto.createHash("sha256").update(data).digest();
  },

  sha512(data: Buffer): Buffer {
    return crypto.createHash("sha512").update(data).digest();
  },

  hmacSha256(key: Buffer, data: Buffer): Buffer {
    return crypto.createHmac("sha256", key).update(data).digest();
  },

  hmacSha512(key: Buffer, data: Buffer): Buffer {
    return crypto.createHmac("sha512", key).update(data).digest();
  },

  randomBytes(n: number): Buffer {
    return crypto.randomBytes(n);
  },

  md5(data: Buffer): Buffer {
    return crypto.createHash("md5").update(data).digest();
  },

  sha1(data: Buffer): Buffer {
    return crypto.createHash("sha1").update(data).digest();
  },

  sha384(data: Buffer): Buffer {
    return crypto.createHash("sha384").update(data).digest();
  },

  sha3_256(data: Buffer): Buffer {
    return crypto.createHash("sha3-256").update(data).digest();
  },

  sha3_512(data: Buffer): Buffer {
    return crypto.createHash("sha3-512").update(data).digest();
  },

  blake2b(data: Buffer, digestSize: number = 64): Buffer {
    return crypto.createHash("blake2b512").update(data).digest();
  },

  blake2s(data: Buffer, digestSize: number = 32): Buffer {
    return crypto.createHash("blake2s256").update(data).digest();
  },

  pbkdf2(password: Buffer, salt: Buffer, iterations: number, keylen: number): Buffer {
    return crypto.pbkdf2Sync(password, salt, iterations, keylen, "sha256");
  },

  scrypt(password: Buffer, salt: Buffer, n: number, r: number, p: number, keylen: number): Buffer {
    return crypto.scryptSync(password, salt, keylen, { N: n, r, p });
  },
};

const TEST_VECTORS: Record<string, Buffer> = {
  empty_input: Buffer.alloc(0),
  single_byte: Buffer.from("x"),
  short_string: Buffer.from("hello world"),
  long_input: Buffer.alloc(10000, 0x61),
  binary_null: Buffer.from([0x00, 0x01, 0x02, 0x00]),
  repetition: Buffer.alloc(256, 0x55),
  boundary_values: Buffer.from(Array.from({ length: 256 }, (_, i) => i)),
  random_input: CryptoBasis.randomBytes(1024),
};

const BASIS_CASES = [
  "empty_input",
  "single_byte",
  "short_string",
  "long_input",
  "binary_null",
  "repetition",
  "boundary_values",
  "random_input",
];

export class CryptoTestHarness {
  public results: TestResult[] = [];
  public testVectors: Record<string, Buffer> = TEST_VECTORS;

  constructor() {
    this.testVectors = { ...TEST_VECTORS };
  }

  private registerResult(name: string, passed: boolean, msg: string, expected: unknown, actual: unknown): void {
    this.results.push([name, passed, msg, expected, actual]);
  }

  public testHashFunction(name: string, func: (data: Buffer) => Buffer): boolean {
    let allPassed = true;
    for (const [caseName, inputData] of Object.entries(this.testVectors)) {
      try {
        const result = func(inputData);
        if (result === null || result.length === 0) {
          this.registerResult(`${name}_${caseName}`, false, "empty result", "non-empty", result);
          allPassed = false;
        } else {
          this.registerResult(`${name}_${caseName}`, true, "ok", result.length, result.length);
        }
      } catch (e) {
        const errMsg = e instanceof Error ? e.message : String(e);
        this.registerResult(`${name}_${caseName}`, false, errMsg, "no error", errMsg);
        allPassed = false;
      }
    }
    return allPassed;
  }

  public testHmacFunction(name: string, func: (key: Buffer, data: Buffer) => Buffer): boolean {
    let allPassed = true;
    const key = CryptoBasis.randomBytes(32);
    for (const [caseName, inputData] of Object.entries(this.testVectors)) {
      try {
        const result = func(key, inputData);
        if (result === null || result.length === 0) {
          this.registerResult(`${name}_${caseName}`, false, "empty result", "non-empty", result);
          allPassed = false;
        } else {
          this.registerResult(`${name}_${caseName}`, true, "ok", result.length, result.length);
        }
      } catch (e) {
        const errMsg = e instanceof Error ? e.message : String(e);
        this.registerResult(`${name}_${caseName}`, false, errMsg, "no error", errMsg);
        allPassed = false;
      }
    }
    return allPassed;
  }

  public testDeterministic(func: (data: Buffer) => Buffer, inputName: string = "test"): boolean {
    const inputData = this.testVectors[inputName];
    const r1 = func(inputData);
    const r2 = func(inputData);
    const isEqual = r1.equals(r2);
    this.registerResult(`deterministic_${inputName}`, isEqual, "consistent output", r1.toString("hex"), r2.toString("hex"));
    return isEqual;
  }

  public testCollisionResistance(func: (data: Buffer) => Buffer, sampleCount: number = 100): boolean {
    const hashes = new Set<string>();
    let allUnique = true;
    for (let i = 0; i < sampleCount; i++) {
      const data = Buffer.from(`test_${i}`);
      const h = func(data).toString("hex");
      if (hashes.has(h)) {
        allUnique = false;
        break;
      }
      hashes.add(h);
    }
    this.registerResult("collision_resistance", allUnique, `${sampleCount} unique`, "unique", allUnique);
    return allUnique;
  }

  public testOutputLengths(func: (data: Buffer) => Buffer, expectedLengths: number[]): boolean {
    const inputData = Buffer.from("test");
    const result = func(inputData);
    if (!expectedLengths.includes(result.length)) {
      this.registerResult("output_length", false, `got ${result.length}`, expectedLengths, result.length);
      return false;
    }
    this.registerResult("output_length", true, "ok", expectedLengths, result.length);
    return true;
  }

  public testKnownAnswer(func: (data: Buffer) => Buffer, inputData: Buffer, expected: Buffer, testName: string): boolean {
    const result = func(inputData);
    const isEqual = result.equals(expected);
    this.registerResult(testName, isEqual, "known answer", expected.toString("hex"), result.toString("hex"));
    return isEqual;
  }

  public runAllTests(): { passed: number; failed: number; total: number } {
    let passed = 0;
    let failed = 0;

    console.log("=".repeat(60));
    console.log("JAVASCRIPT CRYPTOGRAPHY TEST HARNESS");
    console.log("=".repeat(60));

    this.testHashFunction("sha256", CryptoBasis.sha256);
    this.testHashFunction("sha512", CryptoBasis.sha512);
    this.testHashFunction("md5", CryptoBasis.md5);
    this.testHashFunction("sha1", CryptoBasis.sha1);
    this.testHashFunction("sha384", CryptoBasis.sha384);
    this.testHashFunction("sha3_256", CryptoBasis.sha3_256);
    this.testHashFunction("sha3_512", CryptoBasis.sha3_512);
    this.testHashFunction("blake2b", CryptoBasis.blake2b);
    this.testHashFunction("blake2s", CryptoBasis.blake2s);

    this.testHmacFunction("hmac_sha256", CryptoBasis.hmacSha256);
    this.testHmacFunction("hmac_sha512", CryptoBasis.hmacSha512);

    this.testDeterministic(CryptoBasis.sha256, "empty_input");
    this.testDeterministic(CryptoBasis.sha256, "single_byte");
    this.testDeterministic(CryptoBasis.sha256, "short_string");

    this.testCollisionResistance(CryptoBasis.sha256);

    this.testOutputLengths(CryptoBasis.sha256, [32]);
    this.testOutputLengths(CryptoBasis.sha512, [64]);

    const knownAnswers: Array<[string, (data: Buffer) => Buffer, Buffer, Buffer]> = [
      ["sha256", CryptoBasis.sha256, Buffer.alloc(0), Buffer.from("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", "hex")],
      ["sha256", CryptoBasis.sha256, Buffer.from("abc"), Buffer.from("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", "hex")],
      ["sha256", CryptoBasis.sha256, Buffer.from("hello world"), Buffer.from("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9", "hex")],
    ];

    for (const [name, func, inp, exp] of knownAnswers) {
      this.testKnownAnswer(func, inp, exp, `kat_${name}`);
    }

    for (const [name, passedVal, msg, _exp, _act] of this.results) {
      const status = passedVal ? "PASS" : "FAIL";
      console.log(`[${status}] ${name}: ${msg}`);
      if (passedVal) {
        passed++;
      } else {
        failed++;
      }
    }

    console.log("=".repeat(60));
    console.log(`RESULTS: ${passed} passed, ${failed} failed`);
    console.log("=".repeat(60));

    return { passed, failed, total: passed + failed };
  }
}

function main(): number {
  const harness = new CryptoTestHarness();
  const results = harness.runAllTests();

  if (results.failed > 0) {
    console.log(`\nCRITICAL: ${results.failed} tests failed!`);
    return 1;
  }
  console.log("\nAll crypto basis tests passed!");
  return 0;
}

main();
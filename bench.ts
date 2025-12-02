#!/usr/bin/env bun
/**
 * Benchmark script for testing different JS runtimes (qjs, deno, boa, node, bun)
 * Usage: bun bench.ts [-n count] [-v] [runtime...]
 * Example: bun bench.ts qjs deno boa node bun  # test all runtimes, all cases
 *          bun bench.ts -n 10 qjs              # test only qjs with first 10 cases
 *          bun bench.ts -v qjs                 # verbose mode
 */

import { execSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";

const CASES_FILE = "cases.csv";
const PLAYERS_DIR = "players";
const EXE = "ejs";

// Colors
const RED = "\x1b[0;31m";
const GREEN = "\x1b[0;32m";
const YELLOW = "\x1b[1;33m";
const BLUE = "\x1b[0;34m";
const NC = "\x1b[0m";

interface TestCase {
  player: string;
  type: "n" | "sig";
  input: string;
  expected: string;
}

interface TestResult {
  runtime: string;
  passed: number;
  failed: number;
  total: number;
  duration: number;
}

// Parse arguments
let maxTests = 0;
let verbose = false;
const runtimes: string[] = [];

const args = process.argv.slice(2);
for (let i = 0; i < args.length; i++) {
  if (args[i] === "-n" || args[i] === "--count") {
    maxTests = parseInt(args[++i], 10);
  } else if (args[i] === "-v" || args[i] === "--verbose") {
    verbose = true;
  } else {
    runtimes.push(args[i]);
  }
}

if (runtimes.length === 0) {
  runtimes.push("qjs", "deno", "boa", "node", "bun");
}

console.log(`${BLUE}Using executable: ${EXE}${NC}`);
console.log(`${BLUE}Cases file: ${CASES_FILE}${NC}`);
console.log(`${BLUE}Players dir: ${PLAYERS_DIR}${NC}`);
console.log(`${BLUE}Max tests: ${maxTests > 0 ? maxTests : "all"}${NC}`);
console.log("");

// Load test cases
function loadTestCases(): TestCase[] {
  const content = readFileSync(CASES_FILE, "utf-8");
  const cases: TestCase[] = [];

  for (const line of content.split("\n")) {
    if (!line.trim()) continue;

    const parts = line.split(" ");
    if (parts.length < 4) continue;

    const player = parts[0];
    const type = parts[1] as "n" | "sig";
    const input = parts[2];
    const expected = parts[3];

    const playerFile = `${PLAYERS_DIR}/${player}`;
    if (!existsSync(playerFile)) continue;

    cases.push({ player, type, input, expected });
  }

  return cases;
}

// Group test cases by player
function groupByPlayer(cases: TestCase[]): Map<string, TestCase[]> {
  const grouped = new Map<string, TestCase[]>();
  for (const c of cases) {
    if (!grouped.has(c.player)) {
      grouped.set(c.player, []);
    }
    grouped.get(c.player)!.push(c);
  }
  return grouped;
}

// Run tests for a specific runtime
function runTests(runtime: string, allCases: TestCase[]): TestResult {
  let passed = 0;
  let failed = 0;
  let total = 0;
  const errors: string[] = [];

  console.log(`${BLUE}Testing runtime: ${YELLOW}${runtime}${NC}`);
  console.log("----------------------------------------");

  const startTime = performance.now();

  // Apply max tests limit
  const cases = maxTests > 0 ? allCases.slice(0, maxTests) : allCases;
  const grouped = groupByPlayer(cases);

  for (const [player, playerCases] of grouped) {
    const playerFile = `${PLAYERS_DIR}/${player}`;

    // Build arguments
    const args = [playerFile];
    for (const c of playerCases) {
      args.push(`${c.type}:${c.input}`);
    }

    // Run command
    let output = "";
    try {
      output = execSync(`${EXE} --runtime ${runtime} ${args.map(a => `"${a}"`).join(" ")}`, {
        encoding: "utf-8",
        stdio: ["pipe", "pipe", "pipe"],
      });
    } catch (e: any) {
      output = e.stdout || e.stderr || "";
    }

    // Check results
    for (const c of playerCases) {
      total++;
      const expectedStr = `"${c.input}":"${c.expected}"`;

      if (output.includes(expectedStr)) {
        passed++;
      } else {
        failed++;
        const errorMsg = `${RED}FAIL${NC}: ${player} ${c.type}\n  Input: ${c.input}\n  Expected: ${c.expected}`;
        console.error(errorMsg);
        errors.push(`FAIL: ${player} ${c.type}\n  Input: ${c.input}\n  Expected: ${c.expected}`);

        if (verbose) {
          // Try to extract actual result
          const regex = new RegExp(`"${c.input}":"([^"]*)"`, "g");
          const match = regex.exec(output);
          if (match) {
            console.error(`  Actual: ${match[1]}`);
            errors.push(`  Actual: ${match[1]}`);
          }
        }
      }
    }
  }

  const duration = (performance.now() - startTime) / 1000;

  console.log("");
  if (failed === 0) {
    console.log(`${GREEN}Results: ${passed}/${total} passed${NC} (${duration.toFixed(3)}s)`);
  } else {
    console.log(`${YELLOW}Results: ${passed}/${total} passed, ${failed} failed${NC} (${duration.toFixed(3)}s)`);
  }
  console.log("");

  return { runtime, passed, failed, total, duration };
}

// Main
const allCases = loadTestCases();
console.log(`${BLUE}Loaded ${allCases.length} test cases${NC}`);
console.log("");

const results: TestResult[] = [];

for (const runtime of runtimes) {
  const result = runTests(runtime, allCases);
  results.push(result);
}

// Print summary
console.log("========================================");
console.log(`${BLUE}SUMMARY${NC}`);
console.log("========================================");
console.log(
  `${"Runtime".padEnd(10)} ${"Passed".padStart(8)} ${"Failed".padStart(8)} ${"Total".padStart(8)} ${"Time".padStart(12)}`
);
console.log("----------------------------------------");

for (const r of results) {
  const color = r.failed === 0 ? GREEN : YELLOW;
  console.log(
    `${color}${r.runtime.padEnd(10)} ${r.passed.toString().padStart(8)} ${r.failed.toString().padStart(8)} ${r.total.toString().padStart(8)} ${r.duration.toFixed(3).padStart(10)}s${NC}`
  );
}

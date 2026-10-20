#!/usr/bin/env bun
/**
 * Benchmark ejs runtimes.
 *
 * Usage: bun bench.ts [-n count] [-v] [runtime...]
 *   -n, --count N  limit the number of cases
 *   -v, --verbose  print per-player progress and failing commands
 *
 * Set EJS_EXE to benchmark a specific binary.
 */

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";

const CASES_FILE = "cases.csv";
const PLAYERS_DIR = "players";
const EXE =
  process.env.EJS_EXE ??
  (process.platform === "win32" ? "ejs.exe" : "ejs");
const DEFAULT_RUNTIMES = ["qjs", "quickjs-oxide", "deno", "node", "bun"];

const RED = "\x1b[0;31m";
const GREEN = "\x1b[0;32m";
const YELLOW = "\x1b[1;33m";
const BLUE = "\x1b[0;34m";
const BOLD = "\x1b[1m";
const DIM = "\x1b[2m";
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

// ── Arguments ───────────────────────────────────────────────────────────────

let maxTests = 0;
let verbose = false;
const runtimes: string[] = [];
const args = process.argv.slice(2);

for (let i = 0; i < args.length; i++) {
  const arg = args[i];
  if (arg === "-n" || arg === "--count") {
    maxTests = Number.parseInt(args[++i] ?? "0", 10);
  } else if (arg === "-v" || arg === "--verbose") {
    verbose = true;
  } else if (arg === "-h" || arg === "--help") {
    console.log("Usage: bun bench.ts [-n count] [-v] [runtime...]");
    process.exit(0);
  } else {
    runtimes.push(arg);
  }
}

if (!Number.isFinite(maxTests) || maxTests < 0) {
  maxTests = 0;
}
if (runtimes.length === 0) {
  runtimes.push(...DEFAULT_RUNTIMES);
}

// ── Data ────────────────────────────────────────────────────────────────────

function loadTestCases(): TestCase[] {
  const content = readFileSync(CASES_FILE, "utf-8");
  const cases: TestCase[] = [];

  for (const line of content.split("\n")) {
    const parts = line.trim().split(/\s+/);
    if (parts.length < 4) continue;

    const [player, type, input, expected] = parts;
    if (type !== "n" && type !== "sig") continue;
    if (!existsSync(`${PLAYERS_DIR}/${player}`)) continue;

    cases.push({ player, type, input, expected });
  }

  return cases;
}

function groupByPlayer(cases: TestCase[]): Map<string, TestCase[]> {
  const grouped = new Map<string, TestCase[]>();
  for (const testCase of cases) {
    const list = grouped.get(testCase.player);
    if (list) list.push(testCase);
    else grouped.set(testCase.player, [testCase]);
  }
  return grouped;
}

// ── Execution and output parsing ────────────────────────────────────────────

function detectAvailableRuntimes(): Set<string> | null {
  try {
    const help = execFileSync(EXE, ["--help"], {
      encoding: "utf-8",
      stdio: ["ignore", "pipe", "ignore"],
    });
    // clap wraps long help lines, so flatten whitespace before parsing.
    const flat = help.replace(/\s+/g, " ");
    const match = flat.match(/available:\s*([^)]+)\)/i);
    if (!match) return null;
    return new Set(
      match[1]
        .split(",")
        .map((name) => name.trim())
        .filter(Boolean),
    );
  } catch {
    return null;
  }
}

function parseOutput(output: string): {
  results: Map<string, string>;
  error: string | null;
} {
  const results = new Map<string, string>();
  try {
    const parsed = JSON.parse(output);
    if (parsed?.type === "error") {
      return { results, error: String(parsed.error ?? "unknown error") };
    }
    if (parsed?.type === "result" && Array.isArray(parsed.responses)) {
      for (const response of parsed.responses) {
        if (response?.type !== "result" || !response.data) continue;
        for (const [input, value] of Object.entries(response.data)) {
          if (typeof value === "string") results.set(input, value);
        }
      }
    }
  } catch {
    // Non-JSON output is reported through commandError.
  }
  return { results, error: null };
}

function formatDuration(seconds: number): string {
  return `${seconds.toFixed(3)}s`;
}

function pad(value: string | number, width: number, right = false): string {
  const text = String(value);
  return right ? text.padStart(width) : text.padEnd(width);
}

// ── Benchmark ───────────────────────────────────────────────────────────────

function runTests(
  runtime: string,
  grouped: Map<string, TestCase[]>,
  caseCount: number,
  index: number,
  count: number,
): TestResult {
  console.log(
    `${BLUE}${BOLD}[${index}/${count}] Testing ${runtime}${NC} ` +
    `${DIM}(${grouped.size} players, ${caseCount} cases)${NC}`,
  );

  const startTime = performance.now();
  let passed = 0;
  let failed = 0;

  for (const [player, playerCases] of grouped) {
    if (verbose) {
      console.log(
        `${DIM}  ${player} (${playerCases.length} case${playerCases.length === 1 ? "" : "s"
        })${NC}`,
      );
    }

    const commandArgs = [
      "--runtime",
      runtime,
      `${PLAYERS_DIR}/${player}`,
      ...playerCases.map((testCase) => `${testCase.type}:${testCase.input}`),
    ];

    let output = "";
    let commandError = "";
    try {
      output = execFileSync(EXE, commandArgs, {
        encoding: "utf-8",
        stdio: ["ignore", "pipe", "pipe"],
        timeout: 300_000,
        maxBuffer: 16 * 1024 * 1024,
      });
    } catch (error: any) {
      commandError = String(
        error.stderr || error.stdout || error.message || "",
      ).trim();
      if (verbose) {
        console.error(
          `${RED}  command failed:${NC} ${EXE} ${commandArgs.join(" ")}`,
        );
      }
    }

    const { results, error } = parseOutput(output);
    if (error) {
      console.error(`${RED}ejs error${NC} ${player}: ${error}`);
    } else if (commandError && !verbose) {
      console.error(
        `${RED}command error${NC} ${player}: ${commandError.split("\n")[0]}`,
      );
    }

    for (const testCase of playerCases) {
      const actual = results.get(testCase.input);
      if (actual === testCase.expected) {
        passed++;
        continue;
      }

      failed++;
      console.error(
        `${RED}FAIL${NC} ${runtime} | ${player} | ${testCase.type}\n` +
        `  input:    ${testCase.input}\n` +
        `  expected: ${testCase.expected}\n` +
        `  actual:   ${actual ?? "(missing)"}`,
      );
    }
  }

  const duration = (performance.now() - startTime) / 1000;
  const total = passed + failed;
  const perCase = total > 0 ? duration / total : 0;
  const color = failed === 0 ? GREEN : RED;

  console.log(
    `${color}Results: ${passed}/${total} passed, ${failed} failed ` +
    `(${formatDuration(duration)}, ${perCase.toFixed(3)}s/case)${NC}`,
  );
  console.log();

  return { runtime, passed, failed, total, duration };
}

// ── Summary ─────────────────────────────────────────────────────────────────

function printSummary(results: TestResult[], skipped: string[]): void {
  const widths = {
    runtime: Math.max("Runtime".length, ...results.map((r) => r.runtime.length)),
    passed: Math.max("Passed".length, ...results.map((r) => String(r.passed).length)),
    failed: Math.max("Failed".length, ...results.map((r) => String(r.failed).length)),
    total: Math.max("Total".length, ...results.map((r) => String(r.total).length)),
    time: Math.max("Time".length, ...results.map((r) => formatDuration(r.duration).length)),
  };

  const row = (
    runtime: string,
    passed: string | number,
    failed: string | number,
    total: string | number,
    time: string,
  ): string =>
    [
      pad(runtime, widths.runtime),
      pad(passed, widths.passed, true),
      pad(failed, widths.failed, true),
      pad(total, widths.total, true),
      pad(time, widths.time, true),
    ].join("  ");

  const header = row("Runtime", "Passed", "Failed", "Total", "Time");
  const line = "-".repeat(header.length);
  const banner = "=".repeat(header.length);

  console.log(banner);
  console.log(`${BLUE}${BOLD}SUMMARY${NC}`);
  console.log(banner);

  if (results.length === 0) {
    console.log(`${YELLOW}No runtimes were executed.${NC}`);
  } else {
    console.log(`${BOLD}${header}${NC}`);
    console.log(line);

    for (const result of results) {
      const text = row(
        result.runtime,
        result.passed,
        result.failed,
        result.total,
        formatDuration(result.duration),
      );
      const color = result.failed === 0 ? GREEN : RED;
      console.log(`${color}${text}${NC}`);
    }

    const totalPassed = results.reduce((sum, r) => sum + r.passed, 0);
    const totalFailed = results.reduce((sum, r) => sum + r.failed, 0);
    const totalCases = results.reduce((sum, r) => sum + r.total, 0);
    const totalTime = results.reduce((sum, r) => sum + r.duration, 0);
    const color = totalFailed === 0 ? GREEN : RED;
    console.log(line);
    console.log(
      `${color}${BOLD}${row(
        "TOTAL",
        totalPassed,
        totalFailed,
        totalCases,
        formatDuration(totalTime),
      )}${NC}`,
    );
    console.log(line);
  }

  if (skipped.length > 0) {
    console.log(
      `${YELLOW}Skipped (not enabled in ${EXE}): ${skipped.join(", ")}${NC}`,
    );
  }

  const failed =
    results.length === 0 ||
    skipped.length > 0 ||
    results.some((r) => r.failed > 0);
  console.log(failed ? `${RED}FAILED${NC}` : `${GREEN}All runtimes passed.${NC}`);
  if (failed) process.exitCode = 1;
}

// ── Main ────────────────────────────────────────────────────────────────────

const allCases = loadTestCases();
console.log(`${BLUE}Using executable:${NC} ${EXE}`);
console.log(`${BLUE}Cases file:${NC} ${CASES_FILE}`);
console.log(`${BLUE}Players dir:${NC} ${PLAYERS_DIR}`);
console.log(`${BLUE}Max tests:${NC} ${maxTests > 0 ? maxTests : "all"}`);
console.log(`${BLUE}Runtimes:${NC} ${runtimes.join(", ")}`);
console.log();

const available = detectAvailableRuntimes();
if (!available) {
  console.log(
    `${YELLOW}Warning: could not read the runtime list from ${EXE} --help${NC}`,
  );
}

const cases = maxTests > 0 ? allCases.slice(0, maxTests) : allCases;
const grouped = groupByPlayer(cases);
const results: TestResult[] = [];
const skipped: string[] = [];

for (let i = 0; i < runtimes.length; i++) {
  const runtime = runtimes[i];
  if (available && !available.has(runtime)) {
    console.log(`${YELLOW}Skipping ${runtime}: not enabled in ${EXE}${NC}`);
    skipped.push(runtime);
    continue;
  }
  results.push(runTests(runtime, grouped, cases.length, i + 1, runtimes.length));
}

printSummary(results, skipped);

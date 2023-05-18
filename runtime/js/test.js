// A minimalistic test framework for testing Zinnia modules
// Inspired by `node:test`, `Deno.test`, `mocha` and others

import { DenoCore, format_test_error } from "ext:zinnia_runtime/internals.js";

/** @type {{
   pendingTests: TestCase[];
   result: {
    passed: number;
    failed: TestFailure[];
   };
   started: Date;
   lastReportedTest: TestCase | undefined;
 }} */
let globalRoot;

function getGlobalRoot() {
  if (!globalRoot) {
    globalRoot = {
      started: Date.now(),
      pendingTests: [],
      result: {
        passed: 0,
        failed: [],
      },
      lastReportedTest: undefined,
    };
    setTimeout(runNextTest);
  }
  return globalRoot;
}

/** @typedef {{
  fileName: string;
  lineNumber: number;
  columnNumber: number;
}} TestLocation */

class TestCase {
  /**
   * @param {Object} args
   * @param {string} args.name
   * @param {() => void | () => Promise<void>} args.fn
   * @param {TestLocation} args.location
   */
  constructor({ name, fn, location }) {
    this.name = name;
    this.fn = fn;
    this.location = location;
    this.started = undefined;
  }

  async execute() {
    this.started = Date.now();
    let maybePromise;
    try {
      maybePromise = this.fn();
    } catch (error) {
      this.#recordFailure(error);
      return;
    }

    if (typeof maybePromise?.then !== "function") {
      // The test fn was synchronous, we are done.
      this.#recordSuccess();
      return;
    }

    // The test fn returned a promise, we need to wait for completion
    return maybePromise.then(
      () => this.#recordSuccess(),
      (error) => this.#recordFailure(error),
    );
  }

  #log(passed) {
    if (globalRoot.lastReportedTest?.location.fileName !== this.location.fileName) {
      // TODO(bajtos) Ideally, we should report paths relative to the project root
      console.log("\n%s", this.location.fileName.split(/[\\\/]+/g).slice(-1)[0]);
      globalRoot.lastReportedTest = this;
    }

    let duration = Date.now() - this.started;
    console.log(
      "  %s %s %s",
      passed ? green("\u2714" /* check mark */) : red("\u2716" /* heavy multiplication */),
      this.name,
      grey(`(${duration}ms)`),
    );
  }

  #recordFailure(error) {
    globalRoot.result.failed.push(new TestFailure({ testCase: this, error }));
    this.#log(false);
  }

  #recordSuccess() {
    globalRoot.result.passed++;
    this.#log(true);
  }
}

/**
 * @param {TestLocation} location
 * @returns {string}
 */
function displayTestLocation({ fileName, lineNumber, columnNumber }) {
  return `${fileName}:${lineNumber}:${columnNumber}`;
}

class TestFailure {
  /**
   * @param {Object} args
   * @param {TestCase} args.testCase
   * @param {Error} args.error
   */
  constructor({ testCase, error }) {
    this.testCase = testCase;
    this.error = error;
  }

  render() {
    let location = grey(`=> ${displayTestLocation(this.testCase.location)}`);
    return `${this.testCase.name} ${location}\n${boldRed("error:")} ${
      format_test_error(this.error)
      // this.error.stack || this.error
    }`;
  }
}

function runNextTest() {
  const nextTest = globalRoot.pendingTests.shift();

  if (nextTest) {
    nextTest.execute(nextTest).then(runNextTest);
    return;
  }

  reportTestResults();
}

function reportTestResults() {
  let duration = Date.now() - globalRoot.started;
  let failed = globalRoot.result.failed.length;
  let passed = globalRoot.result.passed;

  if (failed) {
    // extra spaces are intentional to show red background
    console.log("\n%s", whiteOnRed(" FAILURES "));
    for (const failure of globalRoot.result.failed) {
      console.log("\n%s", failure.render());
    }
  }

  console.log(
    "\n%s | %s passed | %s failed %s%s",
    failed ? red("FAIL") : green("ok"),
    passed,
    failed,
    grey(`(${duration}ms)`),
    // Add another empty line when all tests passed, for consistency with the failure output
    failed ? "" : "\n",
  );
  // Signal test failure by creating an unhandled error,
  // so that `zinnia` CLI returns a non-zero exit code.
  // TODO(bajtos) Find a more elegant solution, e.g. add an API for setting the exit code
  if (failed) {
    setTimeout(() => {
      const err = new Error();
      // Hack: After the error message, send ASCII sequence to clear the line and
      // move the cursor to column 0. This hides the error message when running in TTY terminals
      // and leaves only the test summary followed by an empty line.
      err.name = `[some tests failed]\u001b[2K\x0D`;
      err.message = undefined;
      err.stack = undefined;
      throw err;
    });
  }
}

export function test(name, fn) {
  if (typeof name !== "string") throw new TypeError(`"name" must be a string, was: ${typeof name}`);
  if (typeof fn !== "function") throw new TypeError(`"fn" must be a function, was: ${typeof fn}`);

  let location = DenoCore.destructureError(new Error()).frames[1];

  getGlobalRoot().pendingTests.push(new TestCase({ name, fn, location }));
}

//
// Helpers for ANSI colors
//
// Useful reference: https://stackoverflow.com/a/33206814/69868
//
function stylize(text, color, style = "") {
  return `\u001b[${style}${color}m${text}\u001b[0m`;
}
function red(text, style = "") {
  return stylize(text, 31, style);
}

function boldRed(text) {
  return red(text, "1;");
}

function green(text) {
  return stylize(text, 32);
}

function grey(text) {
  // white + dim
  return stylize(text, 37, "2;");
}

function whiteOnRed(text) {
  return stylize(text, "37;41");
}

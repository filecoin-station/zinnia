// A dummy wrapper to create isolated scopes for individual tests
// We should eventually replace this with a proper test runner
// See https://github.com/filecoin-station/zinnia/issues/30
let globalRoot;
function getGlobalRoot() {
  if (!globalRoot) {
    globalRoot = {
      pendingTests: [],
      result: {
        passed: 0,
        failed: [],
      },
    };
    Promise.resolve().then(runNextTest);
  }
  return globalRoot;
}

function runNextTest() {
  const nextTest = globalRoot.pendingTests.shift();

  if (nextTest) {
    Promise.resolve(executeTest(nextTest)).then(runNextTest);
    return;
  }

  for (const err of globalRoot.result.failed) {
    console.error(err);
    console.error("\n");
  }

  console.log("\nPASSED: %s FAILED: %s", globalRoot.result.passed, globalRoot.result.failed.length);
  return;
}

function executeTest({ name, fn }) {
  let maybePromise;
  try {
    maybePromise = fn();
  } catch {
    err.message = `Test ${name} failed. ` + err.message;
    globalRoot.result.failed.push(err);
    return;
  }

  if (typeof maybePromise?.then !== "function") {
    // The test fn was synchronous, we are done.
    globalRoot.result.passed++;
    return maybePromise;
  }

  // The test fn returned a promise, we need to wait for completion
  return maybePromise.then(
    (ok) => {
      globalRoot.result.passed++;
      return ok;
    },
    (err) => {
      err.message = `Test ${name} failed. ` + err.message;
      getGlobalRoot().result.failed.push(err);
    },
  );
}

export function test(name, fn) {
  getGlobalRoot().pendingTests.push({ name, fn });
}

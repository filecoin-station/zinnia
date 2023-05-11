// A dummy wrapper to create isolated scopes for individual tests
// We should eventually replace this with a proper test runner
// See https://github.com/filecoin-station/zinnia/issues/30
export function test(name, fn) {
  let maybePromise;
  try {
    maybePromise = fn();
  } catch {
    err.message = `Test ${name} failed. ` + err.message;
    throw err;
  }

  if (typeof maybePromise?.then !== "function") {
    // The test fn was synchronous, we are done.
    return maybePromise;
  }

  // The test fn returned a promise, we need to wait for completion
  return maybePromise.catch((err) => {
    err.message = `Test ${name} failed. ` + err.message;
    throw err;
  });
}

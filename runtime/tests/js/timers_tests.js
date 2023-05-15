const start = Date.now();

await new Promise((resolve) => {
  setTimeout(resolve, 50);
});

const duration = Date.now() - start;
const min = 40;
const max = 120;

if (duration < min || duration > max) {
  throw new Error(
    `setTimeout(50) should take between ${min} to ${max} ms to execute, but took ${duration} ms instead`,
  );
}

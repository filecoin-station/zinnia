const start = Date.now();

await new Promise((resolve) => {
  setTimeout(resolve, 50);
});

const duration = Date.now() - start;

if (duration < 40 || duration > 70) {
  throw new Error(
    `setTimeout(50) should take between 40 to 70 ms to execute, but took ${duration} ms instead`,
  );
}

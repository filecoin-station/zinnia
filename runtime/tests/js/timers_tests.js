const start = Date.now();

await new Promise((resolve, reject) => {
  setTimeout(resolve, 50);
});

const d = Date.now() - start;

if (d < 40 || d > 60) {
  throw new Error(
    `setTimeout(50) should take between 40 to 60 ms to execute, but took ${d} ms instead`,
  );
}

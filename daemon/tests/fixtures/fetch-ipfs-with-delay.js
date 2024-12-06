// Signal that we are going to start the retrieval
Zinnia.activity.info("fetch:start");
const response = await fetch("ipfs://bafybeiazvkej6ou3w6xmva5ed6suonxjv3jkhq4ke73q5hgmcjmf76uos4");

Zinnia.activity.info("fetch:response-headers");

// Introduce some delay before reading the content
await new Promise((resolve) => setTimeout(resolve, 5000));

// Read the content
for await (const _chunk of response.body) {
  // drop the chunk
}

// Signal that the retrieval has finished. If this happens then the test
// did not kill the zinniad process quickly enough.
Zinnia.activity.info("fetch:end");

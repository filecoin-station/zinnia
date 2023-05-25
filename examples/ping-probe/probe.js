// Configuration of InfluxDB writer
const INFLUXDB_API_KEY = "FILL ME IN!";
const INFLUXDB_ORG_ID = "FILL ME IN!";
const INFLUXDB_BUCKET = "FILL ME IN!";
const INFLUXDB_ENDPOINT = "https://eu-central-1-1.aws.cloud2.influxdata.com/";

// List of peers to ping
const PEERS = [
  // Punchr bootstrap nodes
  // https://github.com/libp2p/punchr/blob/b43900e079e654b964531ea6a0b4531c18265b8e/rust-client/src/main.rs#L275-L287
  "/ip4/139.178.91.71/tcp/4001/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
  "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
  "/dnsaddr/bootstrap.libp2p.io/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
  "/dnsaddr/bootstrap.libp2p.io/p2p/QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",

  // ipfs bootstrap nodes
  "/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
  "/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ",
];

// Probe the given peer: send a ping request and measure the time to receive back the response
async function probe(peer) {
  const requestPayload = new Uint8Array(32);
  crypto.getRandomValues(requestPayload);
  const started = Date.now();
  await Zinnia.requestProtocol(peer, "/ipfs/ping/1.0.0", requestPayload);
  const duration = Date.now() - started;
  return { started, duration };
}

// Submit the measured stats to InfluxDB
async function record({ peer, started, duration }) {
  const request_url = new URL("/api/v2/write", INFLUXDB_ENDPOINT);
  request_url.searchParams.set("org", INFLUXDB_ORG_ID);
  request_url.searchParams.set("bucket", INFLUXDB_BUCKET);
  request_url.searchParams.set("precision", "ms");
  const res = await fetch(request_url, {
    method: "POST",
    headers: {
      Accept: "application/json",
      Authorization: `Token ${INFLUXDB_API_KEY}`,
      "Content-Type": "text/plain; charset=utf-8",
    },
    body: `ping_rtt,peer=${peer} rtt=${duration}u ${started}\n`,
  });
  if (!res.ok) {
    throw new Error(`InfluxDB API error ${res.status}\n${await res.text()}`);
  }
}

// Helper function
function sleep(durationInMs) {
  return new Promise((resolve) => setTimeout(resolve, durationInMs));
}

// The main loop
while (true) {
  // 1. Choose a random peer
  const peer = PEERS[Math.floor(Math.random() * PEERS.length)];

  // 2. Run the probe
  let pingResult;
  try {
    console.log("Pinging %s", peer);
    pingResult = await probe(peer);
    console.log("RTT: %sms", pingResult.duration);
  } catch (err) {
    console.error("Cannot ping %s: %s", peer, err);
  }

  // 3. Record the results
  try {
    if (pingResult) {
      await record({ peer, ...pingResult });
      console.log("Submitted stats to InfluxDB.");
    } else {
      // TODO: record ping failure
    }
  } catch (err) {
    console.error("Cannot record stats: %s", err);
  }

  // Notify Filecoin Station
  Zinnia.jobCompleted();

  // Wait one second before running another probe
  await sleep(1000);
}

import { fetch as fetchImpl } from "ext:deno_fetch/26_fetch.js";
import { fromInnerResponse, toInnerResponse } from "ext:deno_fetch/23_response.js";

let ipfsScheme = "ipfs://";
let ipfsBaseUrl = undefined;

export function setLassieUrl(/** @type {string} */ value) {
  ipfsBaseUrl = value + "ipfs/";
}

function rewriteRetrievalUrl(resource) {
  if (!ipfsBaseUrl) {
    throw new Error("Internal Zinnia error: Lassie URL was not configured.");
  }
  if (typeof resource !== "string") resource = String(resource);
  return ipfsBaseUrl + resource.slice(ipfsScheme.length);
}

export function fetch(resource, options) {
  // TODO: support other objects with stringifiers, e.g. URL
  // See https://developer.mozilla.org/en-US/docs/Web/API/fetch#parameters
  if (typeof resource === "string" && resource.startsWith(ipfsScheme)) {
    return fetchFromIpfs(rewriteRetrievalUrl(resource), options, resource);
  }

  // TODO: support `resource` configured as fetch.Request
  // See https://developer.mozilla.org/en-US/docs/Web/API/Request

  return fetchImpl(resource, options);
}

async function fetchFromIpfs(resource, options) {
  // Call Deno's `fetch` using the rewritten URL to make the actual HTTP request
  const response = await fetchImpl(resource, options);

  // Patch the response object to hide the fact that we are calling Lassie
  // We don't want to leak Lassie's URL
  const inner = toInnerResponse(response);
  inner.urlList = inner.urlList.map((url) =>
    url.startsWith(ipfsBaseUrl) ? "ipfs://" + url.slice(ipfsBaseUrl.length) : url,
  );
  return fromInnerResponse(inner);
}

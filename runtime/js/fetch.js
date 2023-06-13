import { fetch as fetchImpl } from "ext:deno_fetch/26_fetch.js";
import { fromInnerResponse, toInnerResponse } from "ext:deno_fetch/23_response.js";
import { toInnerRequest, fromInnerRequest, Request } from "ext:deno_fetch/23_request.js";
import { guardFromHeaders } from "ext:deno_fetch/20_headers.js";

const ipfsScheme = "ipfs://";
let ipfsBaseUrl = undefined;

export function setLassieUrl(/** @type {string} */ value) {
  ipfsBaseUrl = value + "ipfs/";
}

export function fetch(resource, options) {
  let request = new Request(resource, options);
  // The `resource` arg can be a string or any other object with a stringifier — including a URL
  // object — that provides the URL of the resource you want to fetch; or a Request object.
  // See https://developer.mozilla.org/en-US/docs/Web/API/fetch#parameters
  // Fortunately, Request's constructor handles the conversions, and Request#url is always a string.
  // See https://developer.mozilla.org/en-US/docs/Web/API/Request/url
  if (request.url.startsWith(ipfsScheme)) {
    return fetchFromIpfs(request);
  } else {
    return fetchImpl(request);
  }
}

async function fetchFromIpfs(request) {
  // Rewrite request URL to use Lassie
  request = buildIpfsRequest(request);

  // Call Deno's `fetch` using the rewritten URL to make the actual HTTP request
  const response = await fetchImpl(request);

  // Patch the response object to hide the fact that we are calling Lassie
  // We don't want to leak Lassie's URL
  return patchIpfsResponse(response);
}

// Deno's Fetch Request is a thin immutable wrapper around InnerRequest. In order to modify the
// request URL, we must convert Request to InnerRequest first, make changes on the inner object,
// and finally convert the InnerRequest back to a new Request instance.
function buildIpfsRequest(request) {
  const inner = toInnerRequest(request);

  inner.urlList = /** @type {(() => string)[]}*/ (inner.urlList).map((urlFn) => {
    const url = urlFn();
    if (!url.startsWith(ipfsScheme)) return urlFn;
    const newUrl = ipfsBaseUrl + url.slice(ipfsScheme.length);
    return () => newUrl;
  });
  inner.urlListProcessed = /** @type {string[]} */ (inner.urlListProcessed).map((url) =>
    url.startsWith(ipfsScheme) ? ipfsBaseUrl + url.slice(ipfsScheme.length) : url,
  );

  return fromInnerRequest(inner, request.signal, guardFromHeaders(request.headers));
}

// Deno's Fetch Response is a thin immutable wrapper around InnerResponse. In order to modify the
// response URL, we must convert Response to InnerResponse first, make changes on the inner object,
// and finally convert the InnerResponse back to a new Response instance.
function patchIpfsResponse(response) {
  const inner = toInnerResponse(response);

  inner.urlList = /** @type {string[])} */ (inner.urlList).map((url) =>
    url.startsWith(ipfsBaseUrl) ? "ipfs://" + url.slice(ipfsBaseUrl.length) : url,
  );

  return fromInnerResponse(inner, guardFromHeaders(response.headers));
}

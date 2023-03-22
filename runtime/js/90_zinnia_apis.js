const primordials = globalThis.__bootstrap.primordials;
const { ObjectDefineProperties, ObjectCreate } = primordials;

const { ops } = globalThis.Deno.core;

import { readOnly } from "ext:zinnia_runtime/06_util.js";
import * as libp2p from "ext:zinnia_libp2p/01_peer.js";

const zinniaNs = ObjectCreate(null);
ObjectDefineProperties(zinniaNs, libp2p.defaultPeerProps);

const activityApi = ObjectCreate(null);
ObjectDefineProperties(activityApi, {
  info: readOnly(reportInfoActivity),
  error: readOnly(reportErrorActivity),
});

ObjectDefineProperties(zinniaNs, {
  activity: readOnly(activityApi),
  jobCompleted: readOnly(reportJobCompleted),
});

function reportInfoActivity(msg) {
  if (typeof msg !== "string") msg = "" + msg;
  ops.op_info_activity(msg);
}

function reportErrorActivity(msg) {
  if (typeof msg !== "string") msg = "" + msg;
  ops.op_info_activity(msg);
}

function reportJobCompleted() {
  ops.op_job_completed();
}

function debugLog(msg) {
  if (typeof msg !== "string") msg = "" + msg;
  ops.op_debug_print(msg);
}

export { zinniaNs, debugLog };

const primordials = globalThis.__bootstrap.primordials;
const { ObjectDefineProperties, ObjectCreate, ObjectFreeze } = primordials;

const { ops } = globalThis.Deno.core;

import { readOnly } from "ext:zinnia_runtime/06_util.js";
import * as libp2p from "ext:zinnia_libp2p/01_peer.js";
import { inspect } from "ext:deno_console/01_console.js";

const versions = {
  zinnia: "",
  v8: "",
};

function setVersions(zinniaVersion, v8Version) {
  versions.zinnia = zinniaVersion;
  versions.v8 = v8Version;

  ObjectFreeze(versions);
}

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
  versions: readOnly(versions),
  inspect: readOnly(inspect),
});

function reportInfoActivity(msg) {
  if (typeof msg !== "string") msg = "" + msg;
  ops.op_info_activity(msg);
}

function reportErrorActivity(msg) {
  if (typeof msg !== "string") msg = "" + msg;
  ops.op_error_activity(msg);
}

function reportJobCompleted() {
  ops.op_job_completed();
}

function log(msg, level) {
  if (typeof msg !== "string") msg = "" + msg;
  ops.op_zinnia_log(msg, level);
}

export { zinniaNs, log, setVersions };

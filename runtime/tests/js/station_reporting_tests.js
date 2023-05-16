// Smoke tests for reporting APIs
//
// We must not use zinnia:test framework here because we want to capture
// the exact content of stdout and stderr streams.

console.log("console.log");
console.error("console.error");
Zinnia.activity.info("activity.info");
Zinnia.activity.error("activity.error");
Zinnia.jobCompleted();

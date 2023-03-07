# Ping probe

This example shows how to build a simple Zinnia module that periodically pings a randomly selected
peer and reports the statistics to InfluxDB.

## Basic Use

Set up an InfluxDB account, create a bucket for storing the stats and an API key with permissions to
write to this bucket.

Modify the InfluxDB configuration in the `probe.js` header using your credentials. You can also change
the list of peers to probe.

Run the probe using the following command:

```
❯ zinnia run probe.js
```

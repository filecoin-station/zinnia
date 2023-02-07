# Building modules

A Station Module is a long-running process that's performing jobs like network
probes, content delivery, and computation.

Zinnia provides a JavaScript runtime with a set of platform APIs allowing
modules to interact with the outside world.

In the long run, we want Zinnia to be aligned with the Web APIs as much as
feasible.

For the shorter term, we are going to take shortcuts to deliver a useful
platform quickly.

## Getting started

If you haven't done so, then install `zinnia` CLI per
[our instructions](../README.md#installation).

Using your favourite text editor, create a file called `module.js` with the
following content:

```js
console.log("Hello universe!");
```

Open the terminal and run the module by using `zinnia run` command:

```
$ zinnia run module.js
Hello universe!
```

## Platform APIs

### console

Zinnia implements most of the `console` Web APIs like `console.log`. You can
find the full list of supported methods in
[Deno docs](https://deno.land/api@v1.30.3?s=Console) and more details about
individual methods in
[MDN web docs](https://developer.mozilla.org/en-US/docs/Web/API/console.)

### More APIs are coming

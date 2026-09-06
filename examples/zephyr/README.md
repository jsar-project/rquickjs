# Zephyr example

This firmware validates rquickjs and QuickJS on a `no_std + alloc` Zephyr host. It exercises two
Zephyr threads sharing a `parallel` runtime, promises and the job queue, SharedArrayBuffer, 8/16/32
bit Atomics, 64-bit BigInt Atomics, and timed `Atomics.wait`.

Zephyr 4.4.2 and SDK 1.0.1 were used for the initial port. The `mps2/an385` QEMU board provides the
same Cortex-M3 Rust target as `qemu_cortex_m3`, with enough RAM and flash for the engine.

```sh
source /Users/yorkie/zephyrproject/env.sh
cd /Users/yorkie/zephyrproject
west build -b mps2/an385 /Users/yorkie/workspace/rquickjs/examples/zephyr \
  -d build/rquickjs-zephyr
west build -d build/rquickjs-zephyr -t run
```

Exit QEMU with Ctrl+A, then X.

#![no_std]

extern crate alloc;

use rquickjs::{Context, Runtime};
use zephyr::printkln;

const WORKER_STACK_SIZE: usize = 65536;

fn run_engine(rt: &Runtime) -> rquickjs::Result<i32> {
    printkln!("rquickjs: create context");
    rt.set_can_block(true);
    let ctx = Context::full(rt)?;

    ctx.with(|ctx| {
        ctx.eval::<(), _>(
            r#"
            globalThis.promiseResult = 0;
            Promise.resolve(40).then(value => { globalThis.promiseResult = value + 2; });
            "#,
        )
    })?;
    printkln!("rquickjs: promise queued");

    while rt.is_job_pending() {
        rt.execute_pending_job()
            .map_err(|_| rquickjs::Error::Exception)?;
    }
    printkln!("rquickjs: jobs drained");

    ctx.with(|ctx| {
        let result = ctx.eval(
            r#"
            (() => {
                const sab = new SharedArrayBuffer(32);
                const u8 = new Uint8Array(sab);
                const i16 = new Int16Array(sab);
                const i32 = new Int32Array(sab);
                const i64 = new BigInt64Array(sab);

                if (Atomics.add(i32, 0, 40) !== 0) return -1;
                if (Atomics.compareExchange(i32, 0, 40, 42) !== 40) return -2;
                if (Atomics.load(i32, 0) !== 42) return -3;
                if (Atomics.wait(i32, 1, 0, 1) !== "timed-out") return -4;
                Atomics.store(i64, 1, 7n);
                if (Atomics.load(i64, 1) !== 7n) return -5;
                if (Atomics.exchange(u8, 16, 5) !== 0) return -6;
                if (Atomics.or(u8, 16, 2) !== 5) return -7;
                if (Atomics.and(u8, 16, 6) !== 7) return -8;
                if (Atomics.store(i16, 9, 9) !== 9) return -9;
                if (Atomics.sub(i16, 9, 2) !== 9) return -10;
                if (Atomics.xor(i16, 9, 5) !== 7) return -11;
                if (Atomics.add(i64, 1, 2n) !== 7n) return -12;
                if (Atomics.load(i64, 1) !== 9n) return -13;
                if (Atomics.notify(i32, 1, 1) !== 0) return -14;
                Atomics.pause();
                return globalThis.promiseResult;
            })()
            "#,
        );
        if result.is_err() {
            printkln!("rquickjs: JavaScript exception: {:?}", ctx.catch());
        }
        result
    })
}

#[zephyr::thread(stack_size = WORKER_STACK_SIZE, pool_size = 2)]
fn worker(id: usize, runtime: Runtime) {
    printkln!("rquickjs worker {}: start", id);
    match run_engine(&runtime) {
        Ok(42) => printkln!("rquickjs worker {}: PASS", id),
        Ok(value) => panic!("rquickjs worker {} returned {}", id, value),
        Err(error) => panic!("rquickjs worker {} failed: {:?}", id, error),
    }
}

#[no_mangle]
extern "C" fn rust_main() {
    printkln!("rquickjs: create runtime");
    let runtime = Runtime::new().expect("create QuickJS runtime");
    printkln!("rquickjs: runtime ready");

    let first = worker(1, runtime.clone()).start();
    printkln!("rquickjs: first worker started");
    let second = worker(2, runtime).start();
    printkln!("rquickjs: second worker started");
    first.join().expect("join first rquickjs worker");
    second.join().expect("join second rquickjs worker");

    printkln!("rquickjs Zephyr no_std + alloc: PASS");
}

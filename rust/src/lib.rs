// pub use vg_derive::game;
// use wasm_bindgen::prelude::*;

use std::{collections::VecDeque, future::Future, time::Duration};
mod conversions;
mod executor;
pub use conversions::{Position, Rotation};
pub mod gfx;

#[macro_export]
macro_rules! game {
    ($($e: stmt);*) => {
        fn main() {
            #[allow(redundant_semicolons)]
            async fn __vg_wrapper() {
                $(
                    $e
                );*
            }

            __vg_start(__vg_wrapper)
        }
    };
}

// #[no_mangle]

#[cfg(target_arch = "wasm32")]
fn ensure() -> &'static mut State {
    unsafe { STATE.get_or_insert_with(|| unreachable!()) }
}

// This is what happens when you don't use cargo-vg
#[cfg(not(target_arch = "wasm32"))]
fn ensure() -> &'static mut State {
    let mut code = Some(vg_builder::WASM.to_vec());
    vg_native::Engine::run::<vg_native::runtime::wasm::Wasm, _>(move || code.take())
}

static mut STATE: Option<State> = None;

pub struct State {
    tick: usize,
    exec: executor::Executor,
    responses: VecDeque<Vec<u8>>,
    runtime: Duration,
}

#[allow(unused)]
#[link(wasm_import_module = "env")]
extern "C" {
    fn call(ptr: u64, len: u64);
}

pub fn __vg_start<F, Fut>(f: F)
where
    F: Fn() -> Fut,
    Fut: Future<Output = ()> + 'static,
{
    unsafe {
        STATE.get_or_insert_with(|| {
            let exec = executor::Executor::new(f());
            let tick = 0;
            let responses = VecDeque::new();

            State {
                exec,
                tick,
                responses,
                runtime: Duration::from_secs(0),
            }
        });
    }

    ensure();
}

#[no_mangle]
pub extern "C" fn __vg_tick(time: f64) {
    let state = ensure();
    state.tick += 1;
    state.runtime += Duration::from_secs_f64(time);

    state.exec.run();
}

/// Give the host some way to allocate new memory in the client
#[no_mangle]
pub extern "C" fn __vg_allocate(len: u64) -> u64 {
    let resp = &mut ensure().responses;

    resp.push_back(vec![0; len as usize]);
    resp.back().unwrap().as_ptr() as u64
}

#[allow(unused)]
fn call_host(val: impl vg_types::SerBin) {
    ensure();
    #[cfg(target_arch = "wasm32")]
    unsafe {
        let bytes = val.serialize_bin();
        call(bytes.as_ptr() as u64, bytes.len() as u64)
    }
}

// Public api

#[allow(unused)]
pub fn time() -> f64 {
    ensure().runtime.as_secs_f64()
}

#[allow(unused)]
pub fn print(s: impl ToString) {
    call_host(vg_types::Call::Print(s.to_string()))
}

pub async fn frame() {
    call_host(vg_types::Call::Present);
    ensure().exec.halt().await;
}

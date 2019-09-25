// taken from https://github.com/Gigoteur/UnicornConsole/blob/ad62c6a9736fde999629a0e764c4f22140102593/unicorn/src/unicorn/emscripten.rs
#![allow(dead_code)]

#[cfg(not(target_os = "emscripten"))]
compile_error!("This backend only works on Emscripten, please build with `cargo web build`");

#[cfg(target_os = "emscripten")]
pub mod emscripten {
    use std::cell::RefCell;
    use std::os::raw::{c_float, c_int, c_void};
    use std::ptr::null_mut;

    #[allow(non_camel_case_types)]
    type em_callback_func = unsafe extern "C" fn();

    extern "C" {
        // void emscripten_set_main_loop(em_callback_func func, int fps, int simulate_infinite_loop)
        pub fn emscripten_set_main_loop(
            func: em_callback_func,
            fps: c_int,
            simulate_infinite_loop: c_int,
        );

        pub fn emscripten_cancel_main_loop();
        pub fn emscripten_get_now() -> c_float;
    }

    thread_local!(static MAIN_LOOP_CALLBACK: RefCell<*mut c_void> = RefCell::new(null_mut()));

    pub fn set_main_loop_callback<F>(callback: F)
    where
        F: FnMut(),
    {
        MAIN_LOOP_CALLBACK.with(|log| {
            *log.borrow_mut() = &callback as *const _ as *mut c_void;
        });

        unsafe {
            emscripten_set_main_loop(wrapper::<F>, 0, 1);
        }

        unsafe extern "C" fn wrapper<F>()
        where
            F: FnMut(),
        {
            MAIN_LOOP_CALLBACK.with(|z| {
                let closure = *z.borrow_mut() as *mut F;
                (*closure)();
            });
        }
    }
}

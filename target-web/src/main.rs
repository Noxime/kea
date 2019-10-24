#![feature(set_stdio)]

use std::rc::Rc;
use std::sync::Mutex;
use stdweb::console;
use stdweb::unstable::TryInto;
use stdweb::web::{self, html_element::CanvasElement, INode};

mod gfx;

// The default allocator is quite big so this will make release binaries
// smaller as size is a proper issue on the web
#[cfg_attr(not(debug_assertions), global_allocator)]
#[cfg(not(debug_assertions))]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct Kea {
    gfx: gfx::Gfx,
    sfx: placeholder_audio::Audio,
    input: placeholder_input::Input,
}

impl kea::Api for Kea {
    type R = gfx::Gfx;
    type I = placeholder_input::Input;
    type A = placeholder_audio::Audio;

    fn poll(&mut self) {}
    fn exit(&self) -> bool {
        false
    }

    fn audio(&mut self) -> &mut Self::A {
        &mut self.sfx
    }

    fn input(&mut self) -> &mut Self::I {
        &mut self.input
    }

    fn renderer(&mut self) -> &mut Self::R {
        &mut self.gfx
    }
}

pub fn main() {
    // web panics are garbage by default
    std::panic::set_hook(Box::new(|info| {
        console!(error, format!("{}", info));
    }));

    console!(log, "Kea start");

    let document = web::document();
    document.set_title("Kea");
    let element = document.create_element("canvas").unwrap();
    let canvas: CanvasElement = element.try_into().unwrap();
    let body = document.body().unwrap();
    body.append_child(&canvas);
    canvas.set_width(800);
    canvas.set_height(600);

    let (gfx, waker) = gfx::Gfx::new(canvas);

    let kea = Kea {
        gfx,
        sfx: placeholder_audio::Audio::new(),
        input: placeholder_input::Input
    };

    use futures::executor::LocalPool;
    use futures::task::LocalSpawn;
    let executor = LocalPool::new();

    executor
        .spawner()
        .spawn_local_obj(Box::new(game::run(kea)).into())
        .expect("Failed to spawn");

    fn main_loop(mut executor: LocalPool, waker: Rc<Mutex<Option<std::task::Waker>>>) {
        executor.run_until_stalled();

        if let Some(waker) = waker.lock().expect("failed to lock").take() {
            waker.wake();
        } else {
            console!(error, "lol our waker is gone? yikes");
        }

        web::window().request_animation_frame(move |_| main_loop(executor, waker));
    }

    main_loop(executor, waker);
}

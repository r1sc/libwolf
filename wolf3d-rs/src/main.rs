use glfw::Key;
use window_ctx::WindowContxt;

mod window_ctx;
mod glscreen;

fn intro(window_ctx: &mut WindowContxt) {
    println!("Hello this is intro");

    window_ctx.wait_for_key(&Key::Enter);

    println!("Exiting intro");
}

fn main() {
    let mut window_ctx = WindowContxt::new();

    while !window_ctx.exit_requested() {
        window_ctx.poll_events();

        intro(&mut window_ctx);

        window_ctx.swap_buffers();
    }
}

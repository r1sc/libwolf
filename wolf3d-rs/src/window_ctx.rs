use std::collections::HashSet;

use glfw::{Action, Context, GlfwReceiver, Key, WindowEvent};

pub struct WindowContxt {
    glfw: glfw::Glfw,
    window: glfw::PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,

    keys_down: HashSet<Key>
}

impl WindowContxt {
    pub fn new() -> Self {
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

        let (mut window, events) = glfw
            .create_window(300, 300, "Hello this is window", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");

        window.set_key_polling(true);
        window.make_current();

        Self {
            glfw,
            window,
            events,
            keys_down: HashSet::new()
        }
    }

    pub fn poll_events(&mut self) {
        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::Key(k, _, Action::Press, _) => {
                    self.keys_down.insert(k);

                    if k == Key::Escape {
                        self.window.set_should_close(true);
                    }
                }
                glfw::WindowEvent::Key(k, _, Action::Release, _) => {
                    self.keys_down.remove(&k);
                }
                _ => {}
            }
        }
    }

    pub fn wait_for_key(&mut self, key: &Key) {
        while !self.keys_down.contains(key) {
            self.poll_events();
        }
    }

    pub fn is_key_down(&self, key: &Key) -> bool {
        self.keys_down.contains(key)
    }

    pub fn exit_requested(&self) -> bool {
        self.window.should_close()
    }

    pub fn swap_buffers(&mut self) {
        self.window.swap_buffers();
    }
}
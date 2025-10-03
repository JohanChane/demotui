#[macro_export]
macro_rules! render {
    () => {
        demotui_shared::frontend::event::NEED_RENDER
            .store(true, std::sync::atomic::Ordering::Relaxed);
    };
    ($cond:expr) => {
        if $cond {
            render!();
        }
    };
}

#[macro_export]
macro_rules! render_and {
    ($cond:expr) => {
        if $cond {
            demotui_shared::render!();
            true
        } else {
            false
        }
    };
}

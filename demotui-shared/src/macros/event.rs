#[macro_export]
macro_rules! succ {
    ($data:expr) => {
        return Ok(yazi_shared::event::Data::from($data))
    };
    () => {
        return Ok(yazi_shared::event::Data::Nil)
    };
}

#[macro_export]
macro_rules! frontend_emit {
    // (Quit($opt:expr)) => {
    //     yazi_shared::event::Event::Quit($opt).emit();
    // };
    (Call($act:ident, $opt:expr)) => {
        demotui_shared::frontend::event::FrontEndEvent::Call(
            demotui_shared::frontend::op::FrontEndOp::$act($opt),
        )
        .emit();
    };
    // (Seq($cmds:expr)) => {
    //     yazi_shared::event::Event::Seq($cmds).emit();
    // };
    (Key($key:expr)) => {
        demotui_shared::frontend::event::FrontEndEvent::Key($key).emit();
    };
    ($event:ident) => {
        demotui_shared::frontend::event::FrontEndEvent::$event.emit();
    };
}

#[macro_export]
macro_rules! backend_emit {
    (Call($act:ident, $opt:expr)) => {
        demotui_shared::backend::event::BackEndEvent::Call(
            demotui_shared::backend::op::BackEndOp::$act($opt),
        )
        .emit();
    };

    ($event:ident) => {
        demotui_shared::event::Event::$event.emit();
    };
}

use std::error::Error;

use x11rb::{
    connection::{Connection, RequestConnection},
    protocol::{
        xproto::{ConnectionExt, GrabMode, ModMask, ChangeWindowAttributesAux, EventMask, Allow},
        Event,
    },
};

use xkbcommon::xkb::{self, Keysym};

use log::{info, debug};

struct OldKeyboard {
    _context: xkb::Context,
    _keymap: xkb::Keymap,
    state: xkb::State,
}

impl OldKeyboard {
    pub fn new(connection: &x11rb::xcb_ffi::XCBConnection) -> Self {
        // Source: https://github.com/rust-x-bindings/toy_xcb/blob/1ad6acd3a48fa953bdaceec4addb384c15cc3a81/src/keyboard.rs

        let xkbver =
            connection.send_trait_request_with_reply(x11rb::protocol::xkb::UseExtensionRequest {
                wanted_major: xkb::x11::MIN_MAJOR_XKB_VERSION,
                wanted_minor: xkb::x11::MIN_MINOR_XKB_VERSION,
            });

        let xkbver = xkbver
            .expect("Send use extension request for xkb")
            .reply()
            .expect("Query reply for xkb extension request");

        assert!(
            xkbver.supported,
            "required xcb-xkb-{}-{} is not supported",
            xkb::x11::MIN_MAJOR_XKB_VERSION,
            xkb::x11::MIN_MINOR_XKB_VERSION
        );

        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let device_id = xkb::x11::get_core_keyboard_device_id(&connection);

        let keymap = xkb::x11::keymap_new_from_device(
            &context,
            &connection,
            device_id,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        );

        let state = xkb::x11::state_new_from_device(&keymap, &connection, device_id);

        OldKeyboard {
            _context: context,
            _keymap: keymap,
            state: state,
        }
    }
}

pub fn main(display_all_events: bool) -> Result<(), Box<dyn Error>> {
    let (x11, screen_num) = x11rb::xcb_ffi::XCBConnection::connect(None)?;

    let keyboard = OldKeyboard::new(&x11);

    let screen = &x11.setup().roots[screen_num];
    info!("Screen: {screen:#?}");

    let root_window = screen.root;
    info!("Root window: {root_window}");

    let focus_cookie = x11
        .get_input_focus()
        .expect("Connection error while requesting focused window.");
    let focused_window = focus_cookie
        .reply()
        .expect("Couldn't get focused window")
        .focus;

    x11.change_window_attributes(
        focused_window,
        &ChangeWindowAttributesAux {
            event_mask: Some(
                EventMask::PROPERTY_CHANGE
                    | EventMask::KEY_PRESS
                    | EventMask::KEY_RELEASE
            ),
            ..Default::default()
        },
    )
    .expect("Registering events on the focused window should work.")
    .check()
    .expect("The check of registering the events should succeed.");

    x11.grab_key(
        true,
        root_window,
        ModMask::ANY,
        0,
        GrabMode::SYNC,
        GrabMode::SYNC,
    )
    .expect("Send grab key request")
    .check()
    .expect("check grab key request.");

    info!("Waiting for grabbed events...");

    // Use this for testing and actually

    let xdo = libxdo::XDo::new(None).expect("xdo lib should work");
    xdo.enter_text("First test", 10).expect("xdo lib should be able to enter text");
    xdo.enter_text("Testing text input 😊", 10).expect("xdo lib should be able to enter text");

    loop {
        let maybe_event = x11.wait_for_event();

        match maybe_event {
            Ok(event) => {
                if display_all_events {
                    debug!("{event:?}");
                }

                match event {
                    Event::KeyPress(event) => {
                        // TODO: Unfortunately key_get_utf8 does not translate
                        // dead keys e. g. which means there will be a need for
                        // a custom translation method that manually translates
                        // these keys
                        // - XK_dead_circumflex
                        // - XK_dead_acute"

                        // TODO: If this ends up being used for translation find a
                        // cleaner way to do this.
                        let keysym = keyboard.state.key_get_one_sym(event.detail.into());
                        let translated = keyboard.state.key_get_utf8(event.detail.into());

                        info!(
                            "{:?} => '{translated}' Press",
                            keysym.name().unwrap_or("Unknown")
                        );

                        if keysym == Keysym::a {
                            x11.allow_events(Allow::ASYNC_KEYBOARD, event.time)
                                .expect("Connection should still be there")
                                .check()
                                .expect("Verify that the request was send");

                            // TODO: Test sending key strokes
                            // Already testing before starting the event loop
                            continue;
                        }

                        x11.allow_events(Allow::REPLAY_KEYBOARD, event.time)
                            .expect("Connection should still be there")
                            .check()
                            .expect("Verify that the request was send");
                    }
                    Event::KeyRelease(event) => {
                        let keysym = keyboard.state.key_get_one_sym(event.detail.into());
                        let translated = keyboard.state.key_get_utf8(event.detail.into());
                        info!(
                            "{:?} => '{translated}' Release",
                            keysym.name().unwrap_or("Unknown")
                        );

                        if keysym == Keysym::a {
                            x11.allow_events(Allow::ASYNC_KEYBOARD, event.time)
                                .expect("Connection should still be there")
                                .check()
                                .expect("Verify that the request was send");
                            continue;
                        }

                        x11.allow_events(Allow::REPLAY_KEYBOARD, event.time)
                            .expect("Connection should still be there")
                            .check()
                            .expect("Verify that the request was send");
                    }
                    _ => (),
                }
            }
            Err(error) => {
                info!("Error: {error:#?}");
            }
        }
    }
}

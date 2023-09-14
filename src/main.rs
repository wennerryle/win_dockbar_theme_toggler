#![windows_subsystem = "windows"]

use std::alloc::System;

#[global_allocator]
static GLOBAL: System = System;

use core::mem::MaybeUninit;
use trayicon::*;
use winapi::um::winuser;
use registry::{ Hive, Security, Data };

fn main() {
    let mut is_light_theme = get_apps_use_light_theme();

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum Events {
        ClickTrayIcon,
    }

    let (s, r) = std::sync::mpsc::channel::<Events>();
    let dark_icon_bytes = include_bytes!("../resource/icon_dark.ico");
    let light_icon_bytes = include_bytes!("../resource/icon_light.ico");

    let light_icon = Icon::from_buffer(light_icon_bytes, None, None).unwrap();
    let dark_icon = Icon::from_buffer(dark_icon_bytes, None, None).unwrap();

    // Needlessly complicated tray icon with all the whistles and bells
    let mut tray_icon = TrayIconBuilder::new()
        .sender(s)
        .icon_from_buffer(dark_icon_bytes)
        .tooltip("Theme toggle")
        .on_click(Events::ClickTrayIcon)
        .build()
        .unwrap();

    std::thread::spawn(move || {
        r.iter().for_each(|m| match m {
            Events::ClickTrayIcon => {
                is_light_theme = get_apps_use_light_theme();

                if is_light_theme {
                    tray_icon.set_icon(&dark_icon).unwrap();
                    change_theme(false);
                } else {
                    tray_icon.set_icon(&light_icon).unwrap();
                    change_theme(true)
                }
            }
        })
    });

    // Your applications message loop. Because all applications require an
    // application loop, you are best served using an `winit` crate.
    loop {
        unsafe {
            let mut msg = MaybeUninit::uninit();
            let bret = winuser::GetMessageA(msg.as_mut_ptr(), 0 as _, 0, 0);
            if bret > 0 {
                winuser::TranslateMessage(msg.as_ptr());
                winuser::DispatchMessageA(msg.as_ptr());
            } else {
                break;
            }
        }
    }
}

fn change_theme(is_light: bool) {
    let key_path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize";

    let key_apps = "AppsUseLightTheme";
    let regkey_apps = Hive::CurrentUser.open(key_path, Security::Write).unwrap();
    let _ = regkey_apps.set_value(key_apps, &Data::U32(if is_light { 1u32 } else { 0u32 }));

    let key_system = "SystemUsesLightTheme";
    let regkey_system = Hive::CurrentUser.open(key_path, Security::Write).unwrap();
    let _ = regkey_system.set_value(key_system, &Data::U32(if is_light { 1u32 } else { 0u32 }));
}

fn get_apps_use_light_theme() -> bool {
    let key_path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize";
    let key = "AppsUseLightTheme";
    let regkey = Hive::CurrentUser.open(key_path, Security::Read).unwrap();
    let value = regkey.value(key).unwrap();

    if let Data::U32(data) = value {
        data == 1u32
    } else {
        false
    }
}
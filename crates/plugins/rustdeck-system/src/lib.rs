#![allow(clippy::trivially_copy_pass_by_ref, reason = "unit used as state")]
#![allow(clippy::unnecessary_wraps)]

use rustdeck_common::{
    Args, actions, decl_action, decl_plugin, decl_variable, export_plugin, variables,
};
use system_shutdown::{reboot, shutdown};

const fn init() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

const fn update(_: &()) {}

fn run_action(_: &(), id: &str, _: &Args) {
    match id {
        "shutdown" => {
            _ = shutdown();
        }
        "reboot" => {
            _ = reboot();
        }
        _ => unreachable!(),
    }
}

fn get_time() -> i64 {
    #[allow(clippy::cast_possible_wrap)]
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |t| t.as_secs()) as i64;
    let offset_in_sec: i64 = chrono::Local::now().offset().local_minus_utc().into();
    timestamp + offset_in_sec
}

fn get_variable(_: &(), id: &str) -> Result<String, String> {
    Ok(match id {
        "time_hours" => ((get_time() / 3600) % 24).to_string(),
        "time_minutes" => ((get_time() / 60) % 60).to_string(),
        "time" => {
            let time = get_time();
            let minutes = (time / 60) % 60;
            let hours = (time / 3600) % 24;
            format!("{hours}:{minutes:02}")
        }
        _ => unreachable!(),
    })
}

export_plugin! {
    decl_plugin! {
        id: "rustdeck_system",
        name: "RustDeck System",
        desc: "System management plugin",
        variables: variables!(
            decl_variable! {
                id: "time",
                desc: "System time (hh:mm)",
                vtype: "string"
            },
            decl_variable! {
                id: "time_hours",
                desc: "System time (hh)",
                vtype: "string"
            },
            decl_variable! {
                id: "time_minutes",
                desc: "System time (mm)",
                vtype: "string"
            },
        ),
        actions: actions!(
            decl_action! {
                id: "shutdown",
                name: "Shutdown",
                desc: "Shutdown the system"
            },
            decl_action! {
                id: "reboot",
                name: "Reboot",
                desc: "Reboot the system"
            },
        ),

        fn_init: init,
        fn_update: update,
        fn_get_variable: get_variable,
        fn_run_action: run_action,
    }
}

#![allow(clippy::trivially_copy_pass_by_ref, reason = "unit used as state")]
#![allow(clippy::unnecessary_wraps)]
#![allow(unsafe_op_in_unsafe_fn)]

use rustdeck_common::{
    Args, Type,
    builder::{Action, PluginBuilder, Variable},
    decorate_fn_get_variable, decorate_fn_init, decorate_fn_run_action, decorate_fn_update,
    export_plugin,
};
use system_shutdown::{reboot, shutdown};

const fn init() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

const fn update(_: &()) {}

fn run_action(_: &(), id: &str, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    match id {
        "shutdown" => {
            shutdown()?;
        }
        "reboot" => {
            reboot()?;
        }
        "execute" => {
            let line = args.get(0).string().to_string();
            let (exec, args) = line.split_once(' ').ok_or("Format error")?;
            std::process::Command::new(exec).args(args.split(' '));
        }
        _ => unreachable!(),
    }

    Ok(())
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
    PluginBuilder::new(
        "rustdeck_system",
        "RustDeck System",
        "System management plugin",
    )
        .init(decorate_fn_init!(init))
        .update(decorate_fn_update!(update))
        .get_variable(decorate_fn_get_variable!(get_variable))
        .run_action(decorate_fn_run_action!(run_action))
        .variable(Variable::new("time", "System time (hh:mm)", Type::String))
        .variable(Variable::new("time_hours", "System time (hh)", Type::String))
        .variable(Variable::new("time_minutes", "System time (mm)", Type::String))
        .action(Action::new("shutdown", "Shutdown", "Shutdown the system"))
        .action(Action::new("reboot", "Reboot", "Reboot the system"))
        .action(
            Action::new("execute", "Execute", "Execute a command")
                .arg("command", "Command", "Command to run", Type::String)
        )
        .build()
        .unwrap()
}

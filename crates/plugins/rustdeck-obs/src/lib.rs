#![allow(clippy::unnecessary_wraps)]
#![allow(unsafe_op_in_unsafe_fn)]

use std::time::Duration;

use rustdeck_common::{
    Args, Type,
    builder::{Action, ConfigOption, PluginBuilder, Variable},
    decorate_fn_get_config_value, decorate_fn_get_enum, decorate_fn_get_variable, decorate_fn_init,
    decorate_fn_run_action, decorate_fn_set_config_value, decorate_fn_update, export_plugin,
};

struct Config {
    host: String,
    port: u16,
    password: Option<String>,
    connect_timeout: Duration,
}

struct PluginState {
    rt: tokio::runtime::Runtime,
    config: Config,
    client: Option<obws::Client>,
}

fn init() -> Result<PluginState, Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    Ok(PluginState {
        rt,
        config: Config {
            host: "localhost".into(),
            port: 4455,
            password: None,
            connect_timeout: obws::client::DEFAULT_CONNECT_TIMEOUT,
        },
        client: None,
    })
}

fn update(state: &mut PluginState) -> Result<(), Box<dyn std::error::Error>> {
    if state.client.is_none() {
        let connect_config = obws::client::ConnectConfig {
            host: &state.config.host,
            port: state.config.port,
            password: state.config.password.as_ref(),
            connect_timeout: state.config.connect_timeout,

            event_subscriptions: None,
            dangerous: None,
            broadcast_capacity: obws::client::DEFAULT_BROADCAST_CAPACITY,
        };

        let client = state
            .rt
            .block_on(obws::Client::connect_with_config(connect_config))?;

        state.client = Some(client);
    }

    Ok(())
}

fn get_variable(state: &PluginState, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = state
        .client
        .as_ref()
        .ok_or_else(|| String::from("Not initialized"))?;

    Ok(match id {
        "scene" => {
            state
                .rt
                .block_on(client.scenes().current_program_scene())?
                .id
                .name
        }
        "profile" => state.rt.block_on(client.profiles().current())?,
        "is_streaming" => state
            .rt
            .block_on(client.streaming().status())?
            .active
            .to_string(),
        "streaming_state" => {
            if state.rt.block_on(client.streaming().status())?.active {
                "online".into()
            } else {
                "offline".into()
            }
        }
        _ => unreachable!(),
    })
}

#[allow(clippy::too_many_lines)]
fn run_action(
    state: &PluginState,
    id: &str,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = state
        .client
        .as_ref()
        .ok_or_else(|| String::from("Not initialized"))?;

    match id {
        "set_filter" => {
            let source_string = args.get(0).string().to_owned();
            let source = source_string.as_str().into();
            let filter = args.get(1).string().to_owned();

            let filters = client.filters();
            let enabled = match args.get(2).string() {
                "on" => true,
                "off" => false,
                "toggle" => !state.rt.block_on(filters.get(source, &filter))?.enabled,
                _ => unreachable!(),
            };

            state
                .rt
                .block_on(filters.set_enabled(obws::requests::filters::SetEnabled {
                    source,
                    filter: &filter,
                    enabled,
                }))?;
        }
        "set_source_visibility" => {
            let scene = args.get(0).string().to_owned();
            let source = args.get(1).string().to_owned();
            let source_state = args.get(2).string().to_owned();

            let item_id =
                state
                    .rt
                    .block_on(client.scene_items().id(obws::requests::scene_items::Id {
                        scene: scene.as_str().into(),
                        source: source.as_str(),
                        ..Default::default()
                    }))?;

            let enabled = match source_state.as_str() {
                "show" => true,
                "hide" => false,
                "toggle" => !state
                    .rt
                    .block_on(client.scene_items().enabled(scene.as_str().into(), item_id))?,
                _ => unreachable!(),
            };

            state.rt.block_on(client.scene_items().set_enabled(
                obws::requests::scene_items::SetEnabled {
                    scene: scene.as_str().into(),
                    item_id,
                    enabled,
                },
            ))?;
        }
        "set_scene" => {
            state.rt.block_on(
                client
                    .scenes()
                    .set_current_program_scene(args.get(0).string()),
            )?;
        }
        "set_streaming" => match args.get(0).string() {
            "toggle" => {
                state.rt.block_on(client.streaming().toggle())?;
            }
            "start" => {
                state.rt.block_on(client.streaming().start())?;
            }
            "stop" => {
                state.rt.block_on(client.streaming().stop())?;
            }
            _ => unreachable!(),
        },
        "set_recording" => match args.get(0).string() {
            "toggle" => {
                state.rt.block_on(client.recording().toggle())?;
            }
            "start" => {
                state.rt.block_on(client.recording().start())?;
            }
            "stop" => {
                state.rt.block_on(client.recording().stop())?;
            }
            _ => unreachable!(),
        },
        "set_virtual_cam" => match args.get(0).string() {
            "toggle" => {
                state.rt.block_on(client.virtual_cam().toggle())?;
            }
            "start" => {
                state.rt.block_on(client.virtual_cam().start())?;
            }
            "stop" => {
                state.rt.block_on(client.virtual_cam().stop())?;
            }
            _ => unreachable!(),
        },
        "set_mute" => {
            let input = args.get(0).string().to_owned();
            let input_state = args.get(1).string().to_owned();

            match args.get(1).string() {
                "toggle" => {
                    state
                        .rt
                        .block_on(client.inputs().toggle_mute(input.as_str().into()))?;
                }
                "mute" | "unmute" => {
                    state.rt.block_on(
                        client
                            .inputs()
                            .set_muted(input.as_str().into(), input_state == "mute"),
                    )?;
                }
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn get_enum(state: &PluginState, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = state
        .client
        .as_ref()
        .ok_or_else(|| String::from("Not initialized"))?;

    Ok(match id {
        "set_filter.source" => {
            let scenes = state.rt.block_on(client.scenes().list())?;

            scenes
                .scenes
                .iter()
                .flat_map(|scene| {
                    state
                        .rt
                        .block_on(client.scene_items().list(scene.id.name.as_str().into()))
                        .unwrap()
                })
                .map(|item| item.source_name)
                .collect::<Vec<String>>()
                .join("\n")
        }
        "set_filter.filter" => state.rt.block_on(async {
            client.filters();
            String::new()
        }),
        "set_filter.state" => String::from("on\noff\ntoggle"),
        "set_streaming.state" | "set_recording.state" | "set_virtual_cam.state" => {
            String::from("start\nstop\ntoggle")
        }
        "set_scene.scene" | "set_source_visibility.scene" => state
            .rt
            .block_on(client.scenes().list())?
            .scenes
            .iter()
            .rev() // NOTE: The order of OBS scenes is reversed (from bottom to top), so they need a reverse
            .map(|s| s.id.name.clone())
            .collect::<Vec<String>>()
            .join("\n"),
        "set_profile.profile" => state
            .rt
            .block_on(client.profiles().list())?
            .profiles
            .join("\n"),
        "set_mute.source" => state
            .rt
            .block_on(client.inputs().list(None))?
            .iter()
            .map(|i| i.id.name.clone())
            .collect::<Vec<String>>()
            .join("\n"),
        "set_mute.state" => String::from("mute\nunmute\ntoggle"),
        "set_source_visibility.state" => String::from("show\nhide\ntoggle"),
        _ => unreachable!(),
    })
}

fn get_config_value(state: &PluginState, id: &str) -> Result<String, String> {
    Ok(match id {
        "host" => state.config.host.clone(),
        "port" => state.config.port.to_string(),
        "password" => state.config.password.clone().unwrap_or_default(),
        "connect_timeout" => state.config.connect_timeout.as_millis().to_string(),
        _ => unreachable!(),
    })
}

fn set_config_value(state: &mut PluginState, id: &str, value: &Args) -> Result<(), String> {
    match id {
        #[allow(clippy::assigning_clones)]
        "host" => state.config.host = value.get(0).string().to_owned(),
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        "port" => state.config.port = value.get(0).int() as u16,
        "password" => state.config.password = Some(value.get(0).string().to_owned()),
        "connect_timeout" => {
            state.config.connect_timeout =
                Duration::from_millis(value.get(0).int().try_into().unwrap());
        }
        _ => unreachable!(),
    }

    Ok(())
}

export_plugin! {
    PluginBuilder::new("rustdeck_obs", "Rustdeck OBS", "A plugin to manage OBS through websocket")
        .init(decorate_fn_init!(init))
        .update(decorate_fn_update!(update))
        .get_variable(decorate_fn_get_variable!(get_variable))
        .run_action(decorate_fn_run_action!(run_action))
        .get_enum(decorate_fn_get_enum!(get_enum))
        .get_config_value(decorate_fn_get_config_value!(get_config_value))
        .set_config_value(decorate_fn_set_config_value!(set_config_value))
        .config_option(ConfigOption::new("host", "Host", "The host of the OBS websocket", Type::String))
        .config_option(ConfigOption::new("port", "Port", "The port of the OBS websocket", Type::Int))
        .config_option(ConfigOption::new("password", "Password", "Websocket password", Type::String))
        .config_option(ConfigOption::new("connect_timeout", "Connect timeout", "Milliseconds", Type::Int))
        .variable(Variable::new("scene", "Scene", Type::String))
        .variable(Variable::new("profile", "Profile", Type::String))
        .variable(Variable::new("is_streaming", "Is Streaming", Type::Bool))
        .variable(Variable::new("streaming_state", "Streaming State", Type::String))
        // .variable(Variable::new("is_recording", "Is Recording", Type::Bool))
        // .variable(Variable::new("recording_state", "Recording State", Type::String))
        .action(
            Action::new("set_filter", "Set filter state", "")
                .arg("source", "Source", "The name of the source", Type::Enum)
                .arg("filter", "Filter", "The name of the filter", Type::String)
                .arg("state", "State", "", Type::Enum)
        )
        .action(
            Action::new("set_source_visibility", "Set source visibility", "")
                .arg("scene", "Scene", "The name of the scene", Type::Enum)
                .arg("source", "Source", "The name of the source", Type::String)
                .arg("state", "State", "The visibility state", Type::Enum)
        )
        .action(
            Action::new("set_scene", "Set scene", "Sets scene (program)")
                .arg("scene", "Scene", "Destination scene name", Type::Enum)
        )
        .action(
            Action::new("set_streaming", "Set streaming state", "")
                .arg("state", "State", "", Type::Enum)
        )
        .action(
            Action::new("set_recording", "Set recording state", "")
                .arg("state", "State", "", Type::Enum)
        )
        .action(
            Action::new("set_virtual_cam", "Set virtual camera state", "")
                .arg("state", "State", "", Type::Enum)
        )
        .action(
            Action::new("set_profile", "Set profile", "Changes current active profile")
                .arg("profile", "Profile", "The name of the profile", Type::Enum)
        )
        .action(
            Action::new("set_mute", "Set mute", "Mute and unmute audio sources")
                .arg("source", "Source", "The name of the source", Type::Enum)
                .arg("state", "State", "The mute state", Type::Enum)
        )
        .build()
        .unwrap()
}

#![allow(clippy::unnecessary_wraps)]
#![allow(unsafe_op_in_unsafe_fn)]

use rustdeck_common::{
    Args, actions, args, decl_action, decl_arg, decl_plugin, decl_variable, export_plugin,
    variables,
};

struct PluginState {
    rt: tokio::runtime::Runtime,
    client: obws::Client,
}

fn init() -> Result<PluginState, Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let client = rt.block_on(obws::Client::connect("localhost", 4455, Some("aaaaaa")))?;

    Ok(PluginState { rt, client })
}

fn update(_: &PluginState) {}

fn get_variable(state: &PluginState, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(match id {
        "scene" => {
            state
                .rt
                .block_on(state.client.scenes().current_program_scene())?
                .id
                .name
        }
        "profile" => state.rt.block_on(state.client.profiles().current())?,
        "is_streaming" => state
            .rt
            .block_on(state.client.streaming().status())?
            .active
            .to_string(),
        "streaming_state" => {
            if state.rt.block_on(state.client.streaming().status())?.active {
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
    match id {
        "set_filter" => {
            let source_string = args.get(0).string().to_owned();
            let source = source_string.as_str().into();
            let filter = args.get(1).string().to_owned();

            let filters = state.client.filters();
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

            let item_id = state.rt.block_on(state.client.scene_items().id(
                obws::requests::scene_items::Id {
                    scene: scene.as_str().into(),
                    source: source.as_str(),
                    ..Default::default()
                },
            ))?;

            let enabled = match source_state.as_str() {
                "show" => true,
                "hide" => false,
                "toggle" => !state.rt.block_on(
                    state
                        .client
                        .scene_items()
                        .enabled(scene.as_str().into(), item_id),
                )?,
                _ => unreachable!(),
            };

            state.rt.block_on(state.client.scene_items().set_enabled(
                obws::requests::scene_items::SetEnabled {
                    scene: scene.as_str().into(),
                    item_id,
                    enabled,
                },
            ))?;
        }
        "set_scene" => {
            state.rt.block_on(
                state
                    .client
                    .scenes()
                    .set_current_program_scene(args.get(0).string()),
            )?;
        }
        "set_streaming" => match args.get(0).string() {
            "toggle" => {
                state.rt.block_on(state.client.streaming().toggle())?;
            }
            "start" => {
                state.rt.block_on(state.client.streaming().start())?;
            }
            "stop" => {
                state.rt.block_on(state.client.streaming().stop())?;
            }
            _ => unreachable!(),
        },
        "set_recording" => match args.get(0).string() {
            "toggle" => {
                state.rt.block_on(state.client.recording().toggle())?;
            }
            "start" => {
                state.rt.block_on(state.client.recording().start())?;
            }
            "stop" => {
                state.rt.block_on(state.client.recording().stop())?;
            }
            _ => unreachable!(),
        },
        "set_virtual_cam" => match args.get(0).string() {
            "toggle" => {
                state.rt.block_on(state.client.virtual_cam().toggle())?;
            }
            "start" => {
                state.rt.block_on(state.client.virtual_cam().start())?;
            }
            "stop" => {
                state.rt.block_on(state.client.virtual_cam().stop())?;
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
                        .block_on(state.client.inputs().toggle_mute(input.as_str().into()))?;
                }
                "mute" | "unmute" => {
                    state.rt.block_on(
                        state
                            .client
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
    Ok(match id {
        "set_filter.source" => {
            let scenes = state.rt.block_on(state.client.scenes().list())?;

            scenes
                .scenes
                .iter()
                .flat_map(|scene| {
                    state
                        .rt
                        .block_on(
                            state
                                .client
                                .scene_items()
                                .list(scene.id.name.as_str().into()),
                        )
                        .unwrap()
                })
                .map(|item| item.source_name)
                .collect::<Vec<String>>()
                .join("\n")
        }
        "set_filter.filter" => state.rt.block_on(async {
            state.client.filters();
            String::new()
        }),
        "set_filter.state" => String::from("on\noff\ntoggle"),
        "set_streaming.state" | "set_recording.state" | "set_virtual_cam.state" => {
            String::from("start\nstop\ntoggle")
        }
        "set_scene.scene" | "set_source_visibility.scene" => state
            .rt
            .block_on(state.client.scenes().list())?
            .scenes
            .iter()
            .rev() // NOTE: The order of OBS scenes is reversed (from bottom to top), so they need a reverse
            .map(|s| s.id.name.clone())
            .collect::<Vec<String>>()
            .join("\n"),
        "set_profile.profile" => state
            .rt
            .block_on(state.client.profiles().list())?
            .profiles
            .join("\n"),
        "set_mute.source" => state
            .rt
            .block_on(state.client.inputs().list(None))?
            .iter()
            .map(|i| i.id.name.clone())
            .collect::<Vec<String>>()
            .join("\n"),
        "set_mute.state" => String::from("mute\nunmute\ntoggle"),
        "set_source_visibility.state" => String::from("show\nhide\ntoggle"),
        _ => unreachable!(),
    })
}

export_plugin! {
    decl_plugin! {
        id: "rustdeck_obs",
        name: "RustDeck OBS",
        desc: "A plugin to manage OBS through websocket",

        variables: variables!(
            decl_variable! {
                id: "scene",
                desc: "Scene",
                vtype: "string",
            },
            decl_variable! {
                id: "profile",
                desc: "Profile",
                vtype: "string",
            },
            decl_variable! {
                id: "is_streaming",
                desc: "Boolean streaming state",
                vtype: "bool",
            },
            decl_variable! {
                id: "streaming_state",
                desc: "Streaming state string",
                vtype: "string",
            },
        ),

        actions: actions!(
            decl_action! {
                id: "set_filter",
                name: "Set filter state",
                desc: "",
                args: args!(
                    decl_arg! {
                        id: "source",
                        name: "Source",
                        desc: "The name of the source",
                        vtype: "enum",
                    },
                    decl_arg! {
                        id: "filter",
                        name: "Filter",
                        desc: "The name of the filter",
                        vtype: "string",
                    },
                    decl_arg! {
                        id: "state",
                        name: "State",
                        desc: "",
                        vtype: "enum",
                    },
                ),
            },
            decl_action! {
                id: "set_source_visibility",
                name: "Set source visibility",
                desc: "",
                args: args!(
                    decl_arg! {
                        id: "scene",
                        name: "Scene",
                        desc: "",
                        vtype: "enum",
                    },
                    decl_arg! {
                        id: "source",
                        name: "Source",
                        desc: "",
                        vtype: "string",
                    },
                    decl_arg! {
                        id: "state",
                        name: "State",
                        desc: "",
                        vtype: "enum",
                    },
                ),
            },
            decl_action! {
                id: "set_scene",
                name: "Set scene",
                desc: "Sets scene for program",
                args: args!(
                    decl_arg! {
                        id: "scene",
                        name: "To",
                        desc: "Destination scene",
                        vtype: "enum",
                    },
                ),
            },
            decl_action! {
                id: "set_streaming",
                name: "Set streaming state",
                desc: "",
                args: args!(
                    decl_arg! {
                        id: "state",
                        name: "State",
                        desc: "",
                        vtype: "enum",
                    },
                ),
            },
            decl_action! {
                id: "set_recording",
                name: "Set recording state",
                desc: "",
                args: args!(
                    decl_arg! {
                        id: "state",
                        name: "State",
                        desc: "",
                        vtype: "enum",
                    },
                ),
            },
            decl_action! {
                id: "set_virtual_cam",
                name: "Set virtual camera state",
                desc: "",
                args: args!(
                    decl_arg! {
                        id: "state",
                        name: "State",
                        desc: "",
                        vtype: "enum",
                    },
                ),
            },
            decl_action! {
                id: "set_profile",
                name: "Set profile",
                desc: "Changes current active profile",
                args: args! (
                    decl_arg! {
                        id: "profile",
                        name: "Profile",
                        desc: "",
                        vtype: "enum",
                    },
                ),
            },
            decl_action! {
                id: "set_mute",
                name: "Set mute",
                desc: "Mute and unmute audio sources",
                args: args!(
                    decl_arg! {
                        id: "source",
                        name: "Source",
                        desc: "",
                        vtype: "enum",
                    },
                    decl_arg! {
                        id: "state",
                        name: "State",
                        desc: "",
                        vtype: "enum",
                    },
                ),
            },
        ),

        fn_init: init,
        fn_update: update,
        fn_get_variable: get_variable,
        fn_run_action: run_action,

        fn_get_enum: get_enum,
    }
}

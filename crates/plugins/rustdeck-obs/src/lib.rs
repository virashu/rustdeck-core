#![allow(clippy::unnecessary_wraps)]

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
fn run_action(state: &PluginState, id: &str, args: &Args) {
    match id {
        "set_filter" => {
            state.rt.block_on(async {
                let source_string = args.get(0).string().to_owned();
                let source = source_string.as_str().into();
                let filter = args.get(1).string().to_owned();

                let filters = state.client.filters();
                let enabled = match args.get(2).string() {
                    "on" => true,
                    "off" => false,
                    "toggle" => !filters.get(source, &filter).await.unwrap().enabled,
                    _ => unreachable!(),
                };
                _ = filters
                    .set_enabled(obws::requests::filters::SetEnabled {
                        source,
                        filter: &filter,
                        enabled,
                    })
                    .await;
            });
        }
        "set_scene" => {
            state.rt.block_on(async {
                _ = state
                    .client
                    .scenes()
                    .set_current_program_scene(args.get(0).string())
                    .await;
            });
        }
        "set_streaming" => {
            state.rt.block_on(async {
                match args.get(0).string() {
                    "toggle" => {
                        _ = state.client.streaming().toggle().await;
                    }
                    "start" => {
                        _ = state.client.streaming().start().await;
                    }
                    "stop" => {
                        _ = state.client.streaming().stop().await;
                    }
                    _ => unreachable!(),
                }
            });
        }
        "set_recording" => {
            state.rt.block_on(async {
                match args.get(0).string() {
                    "toggle" => {
                        _ = state.client.recording().toggle().await;
                    }
                    "start" => {
                        _ = state.client.recording().start().await;
                    }
                    "stop" => {
                        _ = state.client.recording().stop().await;
                    }
                    _ => unreachable!(),
                }
            });
        }
        "set_virtual_cam" => {
            state.rt.block_on(async {
                match args.get(0).string() {
                    "toggle" => {
                        _ = state.client.virtual_cam().toggle().await;
                    }
                    "start" => {
                        _ = state.client.virtual_cam().start().await;
                    }
                    "stop" => {
                        _ = state.client.virtual_cam().stop().await;
                    }
                    _ => unreachable!(),
                }
            });
        }
        "set_mute" => {
            state.rt.block_on(async {
                let input = args.get(0).string().to_owned();
                let input_state = args.get(1).string().to_owned();

                match args.get(1).string() {
                    "toggle" => {
                        _ = state
                            .client
                            .inputs()
                            .toggle_mute(input.as_str().into())
                            .await;
                    }
                    "mute" | "unmute" => {
                        _ = state
                            .client
                            .inputs()
                            .set_muted(input.as_str().into(), input_state == "mute")
                            .await;
                    }
                    _ => unreachable!(),
                }
            });
        }
        _ => unreachable!(),
    }
}

fn get_enum(state: &PluginState, id: &str) -> String {
    match id {
        "set_filter.source" => {
            let scenes = state
                .rt
                .block_on(async { state.client.scenes().list().await.unwrap() });

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
        "set_scene.scene" => state.rt.block_on(async {
            state
                .client
                .scenes()
                .list()
                .await
                .unwrap()
                .scenes
                .iter()
                .rev() // NOTE: The order of OBS scenes is reversed (from bottom to top), so they need a reverse
                .map(|s| s.id.name.clone())
                .collect::<Vec<String>>()
                .join("\n")
        }),
        "set_profile.profile" => state.rt.block_on(async {
            state
                .client
                .profiles()
                .list()
                .await
                .unwrap()
                .profiles
                .join("\n")
        }),
        "set_mute.source" => state.rt.block_on(async {
            state
                .client
                .inputs()
                .list(None)
                .await
                .unwrap()
                .iter()
                .map(|i| i.id.name.clone())
                .collect::<Vec<String>>()
                .join("\n")
        }),
        "set_mute.state" => String::from("mute\nunmute\ntoggle"),
        _ => unreachable!(),
    }
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

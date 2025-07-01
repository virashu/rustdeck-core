use rustdeck_common::{
    Args, actions, args, decl_action, decl_arg, decl_plugin, decl_variable, export_plugin,
    variables,
};

struct PluginState {
    rt: tokio::runtime::Runtime,
    client: obws::Client,
}

fn init() -> PluginState {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let client = rt
        .block_on(obws::Client::connect("localhost", 4455, Some("aaaaaa")))
        .unwrap();

    PluginState { rt, client }
}

fn update(_: &PluginState) {}

fn get_variable(state: &PluginState, id: &str) -> String {
    if id == "scene" {
        state
            .rt
            .block_on(state.client.scenes().current_program_scene())
            .unwrap()
            .id
            .name
    } else {
        unreachable!()
    }
}

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
        "switch_scene" => {
            state.rt.block_on(async {
                _ = state
                    .client
                    .scenes()
                    .set_current_program_scene(args.get(0).string())
                    .await;
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
        "set_filter.action" => String::from("on\noff\ntoggle"),
        "switch_scene.scene" => state.rt.block_on(async {
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
                        id: "action",
                        name: "State",
                        desc: "",
                        vtype: "enum",
                    },
                ),
            },
            decl_action! {
                id: "switch_scene",
                name: "Switch scene",
                desc: "",
                args: args!(
                    decl_arg! {
                        id: "scene",
                        name: "To",
                        desc: "Destination scene",
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

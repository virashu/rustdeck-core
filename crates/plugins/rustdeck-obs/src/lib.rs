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
        .block_on(obws::Client::connect("localhost", 4455, Some("")))
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
        "toggle_filter" => {
            state.rt.block_on(async {
                let filters = state.client.filters();
                let cur = filters.get("Display".into(), "blur").await.unwrap().enabled;
                _ = filters
                    .set_enabled(obws::requests::filters::SetEnabled {
                        source: "Display".into(),
                        filter: "blur",
                        enabled: !cur,
                    })
                    .await;
            });
        }
        "switch_scene" => {
            state.rt.block_on(async {
                _ = state
                    .client
                    .scenes()
                    .set_current_program_scene(args.get(0).string());
            });
        }
        _ => unreachable!(),
    }
}

fn get_enum(state: &PluginState, id: &str) -> String {
    match id {
        "toggle_filter.source" => state.rt.block_on(async {
            state.client.sources();
            String::new()
        }),
        "toggle_filter.filter" => state.rt.block_on(async {
            state.client.filters();
            String::new()
        }),
        "switch_scene.scene" => state.rt.block_on(async {
            state
                .client
                .scenes()
                .list()
                .await
                .unwrap()
                .scenes
                .iter()
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
                id: "toggle_filter",
                name: "Toggle blur filter",
                desc: "",
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
                    }
                ),
            }
        ),

        fn_init: init,
        fn_update: update,
        fn_get_variable: get_variable,
        fn_run_action: run_action,

        fn_get_enum: get_enum,
    }
}

use obws::Client;
use rustdeck_common::{Args, decl_plugin, decl_variable, export_plugin, variables};

struct PluginState {
    rt: tokio::runtime::Runtime,
    client: Client,
}

fn init() -> PluginState {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let client = rt
        .block_on(Client::connect("localhost", 4455, Some("")))
        .unwrap();

    PluginState { rt, client }
}

fn update(_: &PluginState) {}

fn get_variable(state: &PluginState, id: &str) -> String {
    if id == "scene" {
        return state
            .rt
            .block_on(state.client.scenes().current_program_scene())
            .unwrap()
            .id
            .name;
    }
    String::new()
}

fn run_action(_: &PluginState, _: &str, _: &Args) {}

export_plugin! {
    decl_plugin! {
        id: "rustdeck_obs",
        name: "RustDeck OBS",
        desc: "A plugin to manage OBS through websocket",

        variables: variables!(
            decl_variable! {
                id: "scene",
                desc: "Scene",
                vtype: "string"
            }
        ),

        fn_init: init,
        fn_update: update,
        fn_get_variable: get_variable,
        fn_run_action: run_action
    }
}

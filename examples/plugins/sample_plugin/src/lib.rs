#![allow(clippy::unnecessary_wraps)]
#![allow(unsafe_op_in_unsafe_fn)]

use rustdeck_common::{Args, export_plugin};

struct PluginState {
    counter: i32,
}

const fn init() -> Result<PluginState, String> {
    Ok(PluginState { counter: 0 })
}

const fn update(_: &PluginState) {}

fn get_variable(state: &PluginState, id: &str) -> Result<String, String> {
    Ok(if id == "counter" {
        state.counter.to_string()
    } else {
        unreachable!()
    })
}

fn run_action(
    state: &mut PluginState,
    id: &str,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    match id {
        "add" => {
            let amt = args.get(0).int();
            state.counter += amt;
        }
        "increment" => {
            state.counter += 1;
        }
        "clear" => {
            state.counter = 0;
        }
        _ => unreachable!(),
    }

    Ok(())
}

/*
 * Definition using macros
 */

// use rustdeck_common::{
//     actions, args, decl_action, decl_arg, decl_plugin, decl_variable, variables,
// };
//
// export_plugin! {
//     decl_plugin! {
//         id: "plugin_test",
//         name: "Sample Plugin",
//         desc: "A sample plugin",
//         variables: variables!(
//             decl_variable! {
//                 id: "counter",
//                 desc: "Counter",
//                 vtype: "string",
//             },
//         ),
//         actions: actions!(
//             decl_action! {
//                 id: "increment",
//                 name: "Increment",
//                 desc: "Increment counter",
//             },
//             decl_action! {
//                 id: "add",
//                 name: "Add",
//                 desc: "Add value to counter",
//                 args: args!(
//                     decl_arg! {
//                         id: "amount",
//                         name: "Amount",
//                         desc: "Amount",
//                         vtype: "int",
//                     },
//                 ),
//             },
//             decl_action! {
//                 id: "clear",
//                 name: "Clear",
//                 desc: "Set counter value to 0",
//             },
//         ),

//         fn_init: init,
//         fn_update: update,
//         fn_get_variable: get_variable,
//         fn_run_action: run_action,
//     }
// }

/*
 * Definition using builder pattern
 */

use rustdeck_common::{
    Type,
    builder::{Action, PluginBuilder, Variable},
    decorate_fn_get_variable, decorate_fn_init, decorate_fn_run_action, decorate_fn_update,
};

export_plugin! {
    PluginBuilder::new("plugin_test", "Sample Plugin", "A sample plugin")
        .init(decorate_fn_init!(init))
        .update(decorate_fn_update!(update))
        .get_variable(decorate_fn_get_variable!(get_variable))
        .run_action(decorate_fn_run_action!(run_action))

        /* Variables */
        .variable(Variable::new("counter", "Counter", Type::String))

        /* Actions */
        .action(Action::new("increment", "Increment", "Increment counter"))
        .action(
            Action::new("add", "Add", "Add value to counter")
                .arg("amount", "Amount", "Amount", Type::Int)
        )
        .action(Action::new("clear", "Clear", "Set counter value to 0"))

        .build()
        .unwrap()
}

# Plugins

## Examples

Example of a Rust plugin is available in [/examples/plugins/sample_plugin](/examples/plugins/sample_plugin) \
Types and macros are defined in [rustdeck-common](/crates/rustdeck-common) crate

Example of a C plugin is available in [/examples/plugins/sample_c_plugin](/examples/plugins/sample_c_plugin) \
Types are defined in [/include/common.h](/include/common.h) header

## Structure of a Plugin

A plugin struct should have:

- `id` (`const char*`) -- unique identifier
- `name` (`const char*`) -- display name
- `description` (`const char*`) -- full description

- `variables` (`const Variable*[]` -> `const Variable* const*`) \
  A pointer to first element of `NULL`-terminated array of pointers to `Variable` structs. \
  Can be `NULL` to signify that plugin has no variables.

- `actions` (`const Action*[]` -> `const Action* const*`) \
  Similar to variables, a pointer to first element of `NULL`-terminated array of pointers to `Action` structs. \
  Can be `NULL` to signify that plugin has no actions.

- `fn_init` -- pointer to init function, that returns a pointer to _state_ (any pointer).
- `fn_update` -- pointer to update function. Takes _state_ as first argument.
- `fn_get_variable` -- pointer to function, that takes _state_ and _id_ (`const char*`) of a variable, and returns value of variable (`const char*`) (Can only be a string for now).
- `fn_run_action` -- pointer to function, that takes _state_, _id_ (`const char*`) of an action and an array of [Arg](#arg) (`const Arg*`) \
  Length of arg array guaranteed to be of required length, and guaranteed to have types provided in declaration of action.

- `fn_get_enum` -- pointer to pointer to function, used to get option
  Can be `NULL` if there's no `enum` arguments for any of actions.

## Data types (`r#type`/`vtype` fields)

- `bool`
- `int`
- `float`
- `string`
- `enum` -- a special case of `string` for arguments that should have limited options (for example, `on`/`off`, or available scenes).
  Options are queried from plugin using `fn_get_enum` by id in format `<action id>.<arg id>`.
  The function should return string with options separated by _newline_ (`\n`).

## Rust macros for plugin declaration

```rust
decl_plugin! {
  id: /* literal */,
  name: /* literal */,
  desc: /* literal */,

  variables: /* (optional) `variables!` declaration */
  actions: /* (optional) `actions!` declaration */

  /* `T` is type of plugin's state. Can be any type and is not mutated by the loader. */
  fn_init: /* fn() -> Result<T, impl ToString> */
  fn_update: /* fn(state: &mut T) */
  fn_get_variable: /* fn(state: &mut T, id: &str) -> Result<String, impl ToString> */
  fn_run_action: /* fn(state: &mut T, id: &str, args: &Args) -> Result<(), impl ToString> */
}
```

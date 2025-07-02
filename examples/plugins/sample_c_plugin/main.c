#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "../../../include/common.h"

Result
init()
{
  return (Result){ .status = 0 };
}

void
update(void* state)
{
  return;
}

Result
get_variable(void* state, const char* id)
{
  if (!strcmp(id, "a")) {
    return (Result){ .content = strdup("Value of variable A") };
  } else if (!strcmp(id, "b")) {
    return (Result){ .content = strdup("Value of variable B") };
  }

  return (Result){ .status = 1 };
}

void
run_action(void* state, const char* id, const Arg* args)
{
  if (!strcmp(id, "test_action")) {
    printf("Test action called | Arg value: %s\n", args[0].c);
  }
}

// clang-format off
const Plugin somePlugin = {
  .id = "test_plugin_c",
  .name = "C Sample Plugin",
  .desc = "A sample plugin written in C",

  .variables = (const Variable*[]){
    &(Variable){
      .id = "a",
      .desc = "A Variable",
      .type = Int
    },
    &(Variable){
      .id = "b",
      .desc = "B Variable",
      .type = Float
    },
    NULL
  },
  .actions = (const Action*[]){
    &(Action){
      .id = "test_action",
      .name = "Test Action",
      .desc = "A test action",
      .args = (const ActionArg*[]){
        &(ActionArg){
          .id = "arg_1",
          .name = "Arg #1",
          .desc = "A test argument for action",
          .type = String
        },
        NULL
      }
    },
    NULL
  },

  .fn_init = &init,
  .fn_update = &update,
  .fn_get_variable = &get_variable,
  .fn_run_action = &run_action,

  .fn_get_enum = NULL,
};
// clang-format on

EXPORT
const Plugin*
build()
{
  return &somePlugin;
}

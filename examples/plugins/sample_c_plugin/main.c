#include <assert.h>
#include <stdio.h>
#include <stdlib.h>

#include "common.h"

#ifdef _WIN32
#define EXPORT __declspec(dllexport) __stdcall
#else
#define EXPORT
#endif // _WIN32

void*
init()
{
  return NULL;
}

void
update(void* state)
{
  return;
}

char*
get_variable(void* state, char* id)
{
  return NULL;
}

void
run_action(void* state, char* id)
{
  return;
}

const Plugin somePlugin = {
  .id = "test_plugin_c",
  .name = "C Sample Plugin",
  .desc = "A sample plugin written in C",
  .variables =
    (const Variable*[]){
      &(Variable){ .id = "a", .desc = "A Variable", .type = Integer },
      &(Variable){ .id = "b", .desc = "B Variable", .type = Floating },
      NULL },
  .actions =
    (const Action*[]){
      &(Action){ .id = "test_action",
                 .name = "Test Action",
                 .desc = "A test action",
                 .args = (const ActionArg*[]){ &(ActionArg){
                                                 .id = "arg_1",
                                                 .desc = "Arg test",
                                                 .type = String },
                                               NULL } },
      NULL },
  .fn_init = &init,
  .fn_update = &update,
  .fn_get_variable = &get_variable,
  .fn_run_action = &run_action,
};

EXPORT
const void*
build()
{
  return &somePlugin;
}

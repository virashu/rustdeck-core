//
// This is C bindings for creating a Plugin
//

#include <stdbool.h>
#include <stdint.h>

#ifdef _WIN32
#define EXPORT __declspec(dllexport) __stdcall
#else
#define EXPORT
#endif // _WIN32

typedef struct
{
  int status;
  void* content;
} Result;

enum Type
{
  Bool,
  Int,
  Float,
  String,
  Enum,
};

typedef union
{
  const bool* b;
  const int32_t* i;
  const float* f;
  const char* c;
} Arg;

typedef struct
{
  const char* id;
  const char* name;
  const char* desc;
  const enum Type type;
} ActionArg;

typedef struct
{
  const char* id;
  const char* name;
  const char* desc;
  const ActionArg* const* args;
} Action;

typedef struct
{
  const char* id;
  const char* desc;
  const enum Type type;
} Variable;

typedef struct
{
  const char* id;
  const char* name;
  const char* desc;
  const enum Type type;
} ConfigOption;

typedef struct
{
  const char* id;
  const char* name;
  const char* desc;

  const Variable* const* variables;
  const Action* const* actions;
  const ConfigOption* const* config_options;

  Result (*fn_init)(void);
  Result (*fn_update)(void* state);
  Result (*fn_get_variable)(void* state, const char* id);
  Result (*fn_run_action)(void* state, const char* id, const Arg* args);

  /* Optional */
  Result (**fn_get_enum)(void* state, const char* id);
  Result (**fn_get_config_value)(void* state, const char* id);
  Result (**fn_set_config_value)(void* state, const char* id, const Arg* value);
} Plugin;

EXPORT
const Plugin*
build();

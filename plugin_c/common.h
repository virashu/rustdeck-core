#define export __declspec(dllexport)

struct Plugin {
  void *(*init)(void);

  void (*update)(void *state);
  void (*execute_action)(void *state, char *id);
};

#define pName(x) const char *NAME = x;
#define pDesc(x) const char *DESCRIPTION = x;
#define pId(x) const char *ID = x;
#define pActions(x) const char *ACTIONS = "hello";
#define pVars(x) const char *VARIABLES = "";

#define pExport                                                                \
  export const char *get_name() { return NAME; }                               \
  export const char *get_description() { return DESCRIPTION; }                 \
  export const char *get_id() { return ID; }                                   \
  export const char *get_actions() { return ACTIONS; }                         \
  export const char *get_variables() { return VARIABLES; }

#define pData(x, y, z) export const struct Plugin PLUGIN = x, y, z;
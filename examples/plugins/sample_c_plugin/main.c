#include "common.h"

// Plugin info
pName("CLang Plugin")
pDesc("CLang Plugin")
pId("clang_plugin")
pActions("hello")
pVars("")

// Make exports
pExport

// Define our methods
void *init() {
  return 0;
}

void update(void *state) {
  return;
}

void run_action(void *state, char *id) {
  return;
}

char* get_variable(void *state, char *id) {
  return 0;
}

// Export struct
pData {init, update, run_action, get_variable};

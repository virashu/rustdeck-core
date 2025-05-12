#include "common.h"

// Plugin info
pName("CLang Plugin")
pDesc("CLang Plugin")
pId("CLang Plugin")
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

void execute_action(void *state, char *id) {
  return;
}

// Export struct
pData({init, update, execute_action})

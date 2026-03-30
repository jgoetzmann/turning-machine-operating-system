#ifndef TURINGOS_SHELL_H
#define TURINGOS_SHELL_H

#include <stdio.h>
#include <stddef.h>

typedef enum {
    SHELL_CMD_NONE = 0,
    SHELL_CMD_DIR,
    SHELL_CMD_TYPE,
    SHELL_CMD_RUN,
    SHELL_CMD_CC,
    SHELL_CMD_DEL,
    SHELL_CMD_CLS,
    SHELL_CMD_MEM,
    SHELL_CMD_HALT,
    SHELL_CMD_HELP,
    SHELL_CMD_UNKNOWN
} shell_cmd_t;

typedef struct {
    shell_cmd_t last_cmd;
    char arg[129];
    char line[129];
    unsigned int line_len;
    int run_requested;
    unsigned short run_entry;
    int halt_requested;
} shell_t;

void shell_init(shell_t *sh);
void shell_prompt(FILE *out);
shell_cmd_t shell_parse_line(shell_t *sh, const char *line);
void shell_render_result(shell_t *sh, FILE *out, shell_cmd_t cmd, const char *arg);
void shell_line_reset(shell_t *sh);
int shell_line_push_char(shell_t *sh, char ch);
const char *shell_line_buffer(const shell_t *sh);

#endif

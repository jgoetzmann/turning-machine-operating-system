#include "shell.h"

#include <ctype.h>
#include <stddef.h>
#include <string.h>

static int streq(const char *a, const char *b) {
    return strcmp(a, b) == 0;
}

void shell_init(shell_t *sh) {
    if (sh == NULL) {
        return;
    }
    sh->last_cmd = SHELL_CMD_NONE;
    sh->arg[0] = '\0';
}

void shell_prompt(FILE *out) {
    if (out == NULL) {
        return;
    }
    (void)fputs("A> ", out);
}

shell_cmd_t shell_parse_line(shell_t *sh, const char *line) {
    char word[129];
    size_t wi = 0u;
    size_t i = 0u;

    if (sh == NULL || line == NULL) {
        return SHELL_CMD_UNKNOWN;
    }

    sh->arg[0] = '\0';
    while (line[i] != '\0' && isspace((unsigned char)line[i])) {
        i++;
    }
    while (line[i] != '\0' && !isspace((unsigned char)line[i]) && wi < sizeof(word) - 1u) {
        word[wi++] = (char)tolower((unsigned char)line[i]);
        i++;
    }
    word[wi] = '\0';

    while (line[i] != '\0' && isspace((unsigned char)line[i])) {
        i++;
    }
    if (line[i] != '\0') {
        size_t ai = 0u;
        while (line[i] != '\0' && ai < sizeof(sh->arg) - 1u) {
            sh->arg[ai++] = line[i++];
        }
        sh->arg[ai] = '\0';
    }

    if (word[0] == '\0') {
        sh->last_cmd = SHELL_CMD_NONE;
    } else if (streq(word, "dir")) {
        sh->last_cmd = SHELL_CMD_DIR;
    } else if (streq(word, "type")) {
        sh->last_cmd = SHELL_CMD_TYPE;
    } else if (streq(word, "run")) {
        sh->last_cmd = SHELL_CMD_RUN;
    } else if (streq(word, "cc")) {
        sh->last_cmd = SHELL_CMD_CC;
    } else if (streq(word, "del")) {
        sh->last_cmd = SHELL_CMD_DEL;
    } else if (streq(word, "cls")) {
        sh->last_cmd = SHELL_CMD_CLS;
    } else if (streq(word, "mem")) {
        sh->last_cmd = SHELL_CMD_MEM;
    } else if (streq(word, "halt")) {
        sh->last_cmd = SHELL_CMD_HALT;
    } else if (streq(word, "help")) {
        sh->last_cmd = SHELL_CMD_HELP;
    } else {
        sh->last_cmd = SHELL_CMD_UNKNOWN;
    }
    return sh->last_cmd;
}

void shell_render_result(FILE *out, shell_cmd_t cmd, const char *arg) {
    (void)arg;
    if (out == NULL) {
        return;
    }
    switch (cmd) {
        case SHELL_CMD_NONE:
            break;
        case SHELL_CMD_HELP:
            (void)fputs("dir type run cc del cls mem halt help\n", out);
            break;
        case SHELL_CMD_UNKNOWN:
            (void)fputs("?\n", out);
            break;
        default:
            (void)fputs("OK\n", out);
            break;
    }
}

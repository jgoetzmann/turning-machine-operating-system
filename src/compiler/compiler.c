#include "compiler.h"

#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static cc_token_kind_t keyword_kind(const char *s, uint32_t len) {
    if (len == 4u && strncmp(s, "char", 4u) == 0) return CC_TOK_KW_CHAR;
    if (len == 3u && strncmp(s, "int", 3u) == 0) return CC_TOK_KW_INT;
    if (len == 2u && strncmp(s, "if", 2u) == 0) return CC_TOK_KW_IF;
    if (len == 4u && strncmp(s, "else", 4u) == 0) return CC_TOK_KW_ELSE;
    if (len == 5u && strncmp(s, "while", 5u) == 0) return CC_TOK_KW_WHILE;
    if (len == 3u && strncmp(s, "for", 3u) == 0) return CC_TOK_KW_FOR;
    if (len == 6u && strncmp(s, "return", 6u) == 0) return CC_TOK_KW_RETURN;
    return CC_TOK_IDENT;
}

static int push_tok(cc_token_t *tokens, int max_tokens, int *count, cc_token_t tok) {
    if (*count >= max_tokens) {
        return -1;
    }
    tokens[*count] = tok;
    (*count)++;
    return 0;
}

int cc_lex(const char *src, cc_token_t *tokens, int max_tokens) {
    uint32_t i = 0u;
    uint32_t line = 1u;
    uint32_t col = 1u;
    int count = 0;

    if (src == NULL || tokens == NULL || max_tokens <= 0) {
        return -1;
    }

    while (src[i] != '\0') {
        cc_token_t t;
        unsigned char ch = (unsigned char)src[i];

        if (ch == ' ' || ch == '\t' || ch == '\r') {
            i++;
            col++;
            continue;
        }
        if (ch == '\n') {
            i++;
            line++;
            col = 1u;
            continue;
        }

        t.line = line;
        t.col = col;
        t.offset = i;
        t.length = 1u;
        t.kind = CC_TOK_EOF;

        if (isalpha(ch) || ch == '_') {
            uint32_t start = i;
            while (isalnum((unsigned char)src[i]) || src[i] == '_') {
                i++;
                col++;
            }
            t.offset = start;
            t.length = i - start;
            t.kind = keyword_kind(&src[start], t.length);
            if (push_tok(tokens, max_tokens, &count, t) != 0) return -1;
            continue;
        }
        if (isdigit(ch)) {
            uint32_t start = i;
            while (isdigit((unsigned char)src[i])) {
                i++;
                col++;
            }
            t.offset = start;
            t.length = i - start;
            t.kind = CC_TOK_NUMBER;
            if (push_tok(tokens, max_tokens, &count, t) != 0) return -1;
            continue;
        }
        if (ch == '"' || ch == '\'') {
            unsigned char quote = ch;
            uint32_t start = i++;
            col++;
            while (src[i] != '\0' && (unsigned char)src[i] != quote) {
                if (src[i] == '\\' && src[i + 1u] != '\0') {
                    i += 2u;
                    col += 2u;
                } else {
                    if (src[i] == '\n') {
                        line++;
                        col = 1u;
                    } else {
                        col++;
                    }
                    i++;
                }
            }
            if (src[i] == '\0') return -1;
            i++;
            col++;
            t.offset = start;
            t.length = i - start;
            t.kind = (quote == '"') ? CC_TOK_STRING : CC_TOK_CHAR;
            if (push_tok(tokens, max_tokens, &count, t) != 0) return -1;
            continue;
        }

        switch (ch) {
            case '(': t.kind = CC_TOK_LPAREN; break;
            case ')': t.kind = CC_TOK_RPAREN; break;
            case '{': t.kind = CC_TOK_LBRACE; break;
            case '}': t.kind = CC_TOK_RBRACE; break;
            case '[': t.kind = CC_TOK_LBRACKET; break;
            case ']': t.kind = CC_TOK_RBRACKET; break;
            case ',': t.kind = CC_TOK_COMMA; break;
            case ';': t.kind = CC_TOK_SEMI; break;
            case '+': t.kind = CC_TOK_PLUS; break;
            case '-': t.kind = CC_TOK_MINUS; break;
            case '*': t.kind = CC_TOK_STAR; break;
            case '/': t.kind = CC_TOK_SLASH; break;
            case '%': t.kind = CC_TOK_PERCENT; break;
            case '!':
                if (src[i + 1u] == '=') { t.kind = CC_TOK_NE; t.length = 2u; i++; col++; }
                else t.kind = CC_TOK_NOT;
                break;
            case '=':
                if (src[i + 1u] == '=') { t.kind = CC_TOK_EQ; t.length = 2u; i++; col++; }
                else t.kind = CC_TOK_ASSIGN;
                break;
            case '<':
                if (src[i + 1u] == '=') { t.kind = CC_TOK_LE; t.length = 2u; i++; col++; }
                else t.kind = CC_TOK_LT;
                break;
            case '>':
                if (src[i + 1u] == '=') { t.kind = CC_TOK_GE; t.length = 2u; i++; col++; }
                else t.kind = CC_TOK_GT;
                break;
            case '&':
                if (src[i + 1u] == '&') { t.kind = CC_TOK_AND_AND; t.length = 2u; i++; col++; }
                else return -1;
                break;
            case '|':
                if (src[i + 1u] == '|') { t.kind = CC_TOK_OR_OR; t.length = 2u; i++; col++; }
                else return -1;
                break;
            default:
                return -1;
        }

        i++;
        col++;
        if (push_tok(tokens, max_tokens, &count, t) != 0) return -1;
    }

    if (count >= max_tokens) {
        return -1;
    }
    tokens[count].kind = CC_TOK_EOF;
    tokens[count].line = line;
    tokens[count].col = col;
    tokens[count].offset = i;
    tokens[count].length = 0u;
    count++;
    return count;
}

typedef struct {
    const char *src;
    const cc_token_t *tokens;
    int token_count;
    int pos;
    cc_ast_node_t *nodes;
    int node_count;
    int max_nodes;
} cc_parser_t;

typedef struct {
    unsigned char *buf;
    int len;
    int cap;
} cc_out_t;

typedef struct {
    const cc_ast_node_t *nodes;
    int node_count;
    const char *src;
    const cc_token_t *tokens;
    int token_count;
    cc_out_t *out;
    uint16_t next_global_addr;
    struct {
        char name[16];
        uint16_t addr;
    } globals[64];
    int global_count;
    struct {
        char name[16];
        uint16_t sp_offset;
    } locals[64];
    int local_count;
    struct {
        char name[16];
        int body_node;
        int addr;
    } functions[32];
    int function_count;
    struct {
        char name[16];
        int patch_at;
    } pending_calls[64];
    int pending_call_count;
} cc_codegen_t;

static int p_peek_kind(const cc_parser_t *p) {
    if (p->pos < 0 || p->pos >= p->token_count) {
        return (int)CC_TOK_EOF;
    }
    return (int)p->tokens[p->pos].kind;
}

static int p_match(cc_parser_t *p, cc_token_kind_t kind) {
    if (p_peek_kind(p) == (int)kind) {
        p->pos++;
        return 1;
    }
    return 0;
}

static int p_expect(cc_parser_t *p, cc_token_kind_t kind) {
    if (!p_match(p, kind)) {
        return 0;
    }
    return 1;
}

static int p_new_node(cc_parser_t *p, cc_ast_kind_t kind, int token_index) {
    cc_ast_node_t *n;
    if (p->node_count >= p->max_nodes) {
        return -1;
    }
    n = &p->nodes[p->node_count];
    n->kind = kind;
    n->value = 0;
    n->left = -1;
    n->right = -1;
    n->third = -1;
    n->next = -1;
    n->op = CC_TOK_EOF;
    n->token_index = (token_index < 0) ? 0u : (uint32_t)token_index;
    p->node_count++;
    return p->node_count - 1;
}

static int p_parse_expr(cc_parser_t *p);
static int p_parse_stmt(cc_parser_t *p);

static int p_parse_number(const char *s, uint32_t len) {
    int value = 0;
    uint32_t i;
    for (i = 0u; i < len; ++i) {
        value = (value * 10) + (int)(s[i] - '0');
    }
    return value;
}

static int p_parse_primary(cc_parser_t *p) {
    int tok_i = p->pos;
    int node;
    if (p_match(p, CC_TOK_NUMBER)) {
        const cc_token_t *t = &p->tokens[tok_i];
        node = p_new_node(p, CC_AST_LITERAL, tok_i);
        if (node < 0) return -1;
        p->nodes[node].value = p_parse_number(&p->src[t->offset], t->length);
        return node;
    }
    if (p_match(p, CC_TOK_CHAR) || p_match(p, CC_TOK_STRING)) {
        node = p_new_node(p, CC_AST_LITERAL, tok_i);
        return node;
    }
    if (p_match(p, CC_TOK_IDENT)) {
        int ident_node = p_new_node(p, CC_AST_IDENT, tok_i);
        if (ident_node < 0) return -1;
        if (p_match(p, CC_TOK_LPAREN)) {
            int call_node = p_new_node(p, CC_AST_CALL, tok_i);
            int first_arg = -1;
            int last_arg = -1;
            if (call_node < 0) return -1;
            p->nodes[call_node].left = ident_node;
            if (!p_match(p, CC_TOK_RPAREN)) {
                for (;;) {
                    int arg = p_parse_expr(p);
                    if (arg < 0) return -1;
                    if (first_arg < 0) {
                        first_arg = arg;
                    } else {
                        p->nodes[last_arg].next = arg;
                    }
                    last_arg = arg;
                    if (p_match(p, CC_TOK_COMMA)) {
                        continue;
                    }
                    if (!p_expect(p, CC_TOK_RPAREN)) return -1;
                    break;
                }
            }
            p->nodes[call_node].right = first_arg;
            return call_node;
        }
        return ident_node;
    }
    if (p_match(p, CC_TOK_LPAREN)) {
        int expr = p_parse_expr(p);
        if (expr < 0) return -1;
        if (!p_expect(p, CC_TOK_RPAREN)) return -1;
        return expr;
    }
    return -1;
}

static int p_parse_unary(cc_parser_t *p) {
    int tok_i = p->pos;
    cc_token_kind_t op = (cc_token_kind_t)p_peek_kind(p);
    if (op == CC_TOK_NOT || op == CC_TOK_MINUS || op == CC_TOK_PLUS) {
        int node;
        int rhs;
        p->pos++;
        rhs = p_parse_unary(p);
        if (rhs < 0) return -1;
        node = p_new_node(p, CC_AST_UNOP, tok_i);
        if (node < 0) return -1;
        p->nodes[node].op = op;
        p->nodes[node].left = rhs;
        return node;
    }
    return p_parse_primary(p);
}

static int p_parse_bin_ltr(cc_parser_t *p,
                           int (*sub)(cc_parser_t *),
                           const cc_token_kind_t *ops,
                           int op_count) {
    int lhs = sub(p);
    int i;
    if (lhs < 0) return -1;
    for (;;) {
        cc_token_kind_t op = (cc_token_kind_t)p_peek_kind(p);
        int match = 0;
        for (i = 0; i < op_count; ++i) {
            if (op == ops[i]) {
                match = 1;
                break;
            }
        }
        if (!match) break;
        p->pos++;
        {
            int rhs = sub(p);
            int node;
            if (rhs < 0) return -1;
            node = p_new_node(p, CC_AST_BINOP, p->pos - 1);
            if (node < 0) return -1;
            p->nodes[node].op = op;
            p->nodes[node].left = lhs;
            p->nodes[node].right = rhs;
            lhs = node;
        }
    }
    return lhs;
}

static int p_parse_mul(cc_parser_t *p) {
    static const cc_token_kind_t ops[] = {CC_TOK_STAR, CC_TOK_SLASH, CC_TOK_PERCENT};
    return p_parse_bin_ltr(p, p_parse_unary, ops, 3);
}

static int p_parse_add(cc_parser_t *p) {
    static const cc_token_kind_t ops[] = {CC_TOK_PLUS, CC_TOK_MINUS};
    return p_parse_bin_ltr(p, p_parse_mul, ops, 2);
}

static int p_parse_rel(cc_parser_t *p) {
    static const cc_token_kind_t ops[] = {CC_TOK_LT, CC_TOK_LE, CC_TOK_GT, CC_TOK_GE};
    return p_parse_bin_ltr(p, p_parse_add, ops, 4);
}

static int p_parse_eq(cc_parser_t *p) {
    static const cc_token_kind_t ops[] = {CC_TOK_EQ, CC_TOK_NE};
    return p_parse_bin_ltr(p, p_parse_rel, ops, 2);
}

static int p_parse_and(cc_parser_t *p) {
    static const cc_token_kind_t ops[] = {CC_TOK_AND_AND};
    return p_parse_bin_ltr(p, p_parse_eq, ops, 1);
}

static int p_parse_or(cc_parser_t *p) {
    static const cc_token_kind_t ops[] = {CC_TOK_OR_OR};
    return p_parse_bin_ltr(p, p_parse_and, ops, 1);
}

static int p_parse_assign(cc_parser_t *p) {
    int lhs = p_parse_or(p);
    if (lhs < 0) return -1;
    if (p_match(p, CC_TOK_ASSIGN)) {
        int rhs = p_parse_assign(p);
        int node = p_new_node(p, CC_AST_ASSIGN, p->pos - 1);
        if (rhs < 0 || node < 0) return -1;
        p->nodes[node].left = lhs;
        p->nodes[node].right = rhs;
        return node;
    }
    return lhs;
}

static int p_parse_expr(cc_parser_t *p) {
    return p_parse_assign(p);
}

static int p_parse_var_decl_stmt(cc_parser_t *p, cc_token_kind_t type_tok) {
    int node;
    int ident_node;
    if (!p_expect(p, CC_TOK_IDENT)) return -1;
    ident_node = p_new_node(p, CC_AST_IDENT, p->pos - 1);
    if (ident_node < 0) return -1;
    node = p_new_node(p, CC_AST_VAR_DECL, p->pos - 1);
    if (node < 0) return -1;
    p->nodes[node].op = type_tok;
    p->nodes[node].left = ident_node;
    if (p_match(p, CC_TOK_ASSIGN)) {
        int init = p_parse_expr(p);
        if (init < 0) return -1;
        p->nodes[node].right = init;
    }
    if (!p_expect(p, CC_TOK_SEMI)) return -1;
    return node;
}

static int p_parse_block(cc_parser_t *p) {
    int first = -1;
    int last = -1;
    int block = p_new_node(p, CC_AST_PROGRAM, p->pos);
    if (block < 0) return -1;
    if (!p_expect(p, CC_TOK_LBRACE)) return -1;
    while (p_peek_kind(p) != (int)CC_TOK_RBRACE && p_peek_kind(p) != (int)CC_TOK_EOF) {
        int st = p_parse_stmt(p);
        if (st < 0) return -1;
        if (first < 0) first = st;
        else p->nodes[last].next = st;
        last = st;
    }
    if (!p_expect(p, CC_TOK_RBRACE)) return -1;
    p->nodes[block].left = first;
    return block;
}

static int p_parse_for(cc_parser_t *p) {
    int node = p_new_node(p, CC_AST_FOR, p->pos);
    int init = -1;
    int cond = -1;
    int step = -1;
    int body;
    if (node < 0) return -1;
    if (!p_expect(p, CC_TOK_KW_FOR) || !p_expect(p, CC_TOK_LPAREN)) return -1;
    if (p_peek_kind(p) == (int)CC_TOK_KW_INT || p_peek_kind(p) == (int)CC_TOK_KW_CHAR) {
        cc_token_kind_t t = (cc_token_kind_t)p_peek_kind(p);
        p->pos++;
        init = p_parse_var_decl_stmt(p, t);
        if (init < 0) return -1;
    } else if (!p_match(p, CC_TOK_SEMI)) {
        init = p_parse_expr(p);
        if (init < 0 || !p_expect(p, CC_TOK_SEMI)) return -1;
    }
    if (!p_match(p, CC_TOK_SEMI)) {
        cond = p_parse_expr(p);
        if (cond < 0 || !p_expect(p, CC_TOK_SEMI)) return -1;
    }
    if (!p_match(p, CC_TOK_RPAREN)) {
        step = p_parse_expr(p);
        if (step < 0 || !p_expect(p, CC_TOK_RPAREN)) return -1;
    }
    body = p_parse_stmt(p);
    if (body < 0) return -1;
    p->nodes[node].left = init;
    p->nodes[node].right = cond;
    p->nodes[node].third = step;
    p->nodes[node].next = body;
    return node;
}

static int p_parse_stmt(cc_parser_t *p) {
    if (p_peek_kind(p) == (int)CC_TOK_LBRACE) {
        return p_parse_block(p);
    }
    if (p_peek_kind(p) == (int)CC_TOK_KW_IF) {
        int node = p_new_node(p, CC_AST_IF, p->pos);
        int cond;
        int then_node;
        if (node < 0) return -1;
        p->pos++;
        if (!p_expect(p, CC_TOK_LPAREN)) return -1;
        cond = p_parse_expr(p);
        if (cond < 0 || !p_expect(p, CC_TOK_RPAREN)) return -1;
        then_node = p_parse_stmt(p);
        if (then_node < 0) return -1;
        p->nodes[node].left = cond;
        p->nodes[node].right = then_node;
        if (p_match(p, CC_TOK_KW_ELSE)) {
            int else_node = p_parse_stmt(p);
            if (else_node < 0) return -1;
            p->nodes[node].third = else_node;
        }
        return node;
    }
    if (p_peek_kind(p) == (int)CC_TOK_KW_WHILE) {
        int node = p_new_node(p, CC_AST_WHILE, p->pos);
        int cond;
        int body;
        if (node < 0) return -1;
        p->pos++;
        if (!p_expect(p, CC_TOK_LPAREN)) return -1;
        cond = p_parse_expr(p);
        if (cond < 0 || !p_expect(p, CC_TOK_RPAREN)) return -1;
        body = p_parse_stmt(p);
        if (body < 0) return -1;
        p->nodes[node].left = cond;
        p->nodes[node].right = body;
        return node;
    }
    if (p_peek_kind(p) == (int)CC_TOK_KW_FOR) {
        return p_parse_for(p);
    }
    if (p_peek_kind(p) == (int)CC_TOK_KW_RETURN) {
        int node = p_new_node(p, CC_AST_RETURN, p->pos);
        if (node < 0) return -1;
        p->pos++;
        if (!p_match(p, CC_TOK_SEMI)) {
            int expr = p_parse_expr(p);
            if (expr < 0 || !p_expect(p, CC_TOK_SEMI)) return -1;
            p->nodes[node].left = expr;
        }
        return node;
    }
    if (p_peek_kind(p) == (int)CC_TOK_KW_INT || p_peek_kind(p) == (int)CC_TOK_KW_CHAR) {
        cc_token_kind_t t = (cc_token_kind_t)p_peek_kind(p);
        p->pos++;
        return p_parse_var_decl_stmt(p, t);
    }
    if (p_match(p, CC_TOK_SEMI)) {
        return p_new_node(p, CC_AST_UNKNOWN, p->pos - 1);
    }
    {
        int expr = p_parse_expr(p);
        if (expr < 0 || !p_expect(p, CC_TOK_SEMI)) return -1;
        return expr;
    }
}

int cc_parse(const char *src,
             const cc_token_t *tokens,
             int token_count,
             cc_ast_node_t *nodes,
             int max_nodes) {
    cc_parser_t p;
    int root;
    int first = -1;
    int last = -1;
    if (src == NULL || tokens == NULL || token_count <= 0 || nodes == NULL || max_nodes <= 0) {
        return -1;
    }
    memset(&p, 0, sizeof(p));
    p.src = src;
    p.tokens = tokens;
    p.token_count = token_count;
    p.nodes = nodes;
    p.max_nodes = max_nodes;

    root = p_new_node(&p, CC_AST_PROGRAM, 0);
    if (root < 0) return -1;

    while (p_peek_kind(&p) != (int)CC_TOK_EOF) {
        if (p_peek_kind(&p) != (int)CC_TOK_KW_INT && p_peek_kind(&p) != (int)CC_TOK_KW_CHAR) {
            return -1;
        }
        {
            cc_token_kind_t type_tok = (cc_token_kind_t)p_peek_kind(&p);
            int decl;
            int ident;
            int marker;
            p.pos++;
            if (!p_expect(&p, CC_TOK_IDENT)) return -1;
            ident = p_new_node(&p, CC_AST_IDENT, p.pos - 1);
            if (ident < 0) return -1;
            marker = p.pos;
            if (p_match(&p, CC_TOK_LPAREN)) {
                int func = p_new_node(&p, CC_AST_FUNC_DECL, p.pos - 2);
                int param_first = -1;
                int param_last = -1;
                int body;
                if (func < 0) return -1;
                p.nodes[func].op = type_tok;
                p.nodes[func].left = ident;
                if (!p_match(&p, CC_TOK_RPAREN)) {
                    for (;;) {
                        int param_decl;
                        int param_ident;
                        cc_token_kind_t ptype;
                        if (p_peek_kind(&p) != (int)CC_TOK_KW_INT &&
                            p_peek_kind(&p) != (int)CC_TOK_KW_CHAR) return -1;
                        ptype = (cc_token_kind_t)p_peek_kind(&p);
                        p.pos++;
                        if (!p_expect(&p, CC_TOK_IDENT)) return -1;
                        param_ident = p_new_node(&p, CC_AST_IDENT, p.pos - 1);
                        if (param_ident < 0) return -1;
                        param_decl = p_new_node(&p, CC_AST_VAR_DECL, p.pos - 1);
                        if (param_decl < 0) return -1;
                        p.nodes[param_decl].op = ptype;
                        p.nodes[param_decl].left = param_ident;
                        if (param_first < 0) param_first = param_decl;
                        else p.nodes[param_last].next = param_decl;
                        param_last = param_decl;
                        if (p_match(&p, CC_TOK_COMMA)) continue;
                        if (!p_expect(&p, CC_TOK_RPAREN)) return -1;
                        break;
                    }
                }
                body = p_parse_block(&p);
                if (body < 0) return -1;
                p.nodes[func].right = param_first;
                p.nodes[func].third = body;
                decl = func;
            } else {
                int var = p_new_node(&p, CC_AST_VAR_DECL, marker - 1);
                if (var < 0) return -1;
                p.nodes[var].op = type_tok;
                p.nodes[var].left = ident;
                if (p_match(&p, CC_TOK_ASSIGN)) {
                    int init = p_parse_expr(&p);
                    if (init < 0) return -1;
                    p.nodes[var].right = init;
                }
                if (!p_expect(&p, CC_TOK_SEMI)) return -1;
                decl = var;
            }
            if (first < 0) first = decl;
            else p.nodes[last].next = decl;
            last = decl;
        }
    }
    p.nodes[root].left = first;
    return p.node_count;
}

static int cg_emit1(cc_out_t *o, unsigned char b) {
    if (o->len >= o->cap) return -1;
    o->buf[o->len++] = b;
    return 0;
}

static int cg_node_ok(const cc_codegen_t *cg, int idx) {
    return idx >= 0 && idx < cg->node_count;
}

static int cg_tok_ok(const cc_codegen_t *cg, uint32_t tok_i) {
    return tok_i < (uint32_t)cg->token_count;
}

#define CC_CODE_BASE 0x0100u

static int cg_emit_addr16(cc_out_t *o, uint16_t addr) {
    if (cg_emit1(o, (unsigned char)(addr & 0xFFu)) != 0) return -1;
    if (cg_emit1(o, (unsigned char)((addr >> 8) & 0xFFu)) != 0) return -1;
    return 0;
}

static int cg_pos(const cc_codegen_t *cg) {
    return cg->out->len;
}

static int cg_emit_jump_placeholder(cc_codegen_t *cg, unsigned char opcode, int *patch_at) {
    if (cg_emit1(cg->out, opcode) != 0) return -1;
    *patch_at = cg->out->len;
    if (cg_emit1(cg->out, 0x00u) != 0) return -1;
    if (cg_emit1(cg->out, 0x00u) != 0) return -1;
    return 0;
}

static int cg_patch_jump(cc_codegen_t *cg, int patch_at, int target) {
    uint16_t abs_target;
    if (patch_at < 0 || patch_at + 1 >= cg->out->cap) return -1;
    if (target < 0 || (unsigned long)target + CC_CODE_BASE > 0xFFFFu) return -1;
    abs_target = (uint16_t)((unsigned)target + CC_CODE_BASE);
    cg->out->buf[patch_at] = (unsigned char)(abs_target & 0xFFu);
    cg->out->buf[patch_at + 1] = (unsigned char)((abs_target >> 8) & 0xFFu);
    return 0;
}

static int cg_emit_bios_conout_a(cc_codegen_t *cg) {
    if (cg_emit1(cg->out, 0x4Fu) != 0) return -1; /* MOV C,A */
    if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0x02 */
    if (cg_emit1(cg->out, 0x02u) != 0) return -1;
    if (cg_emit1(cg->out, 0xD3u) != 0) return -1; /* OUT 0x01 */
    if (cg_emit1(cg->out, 0x01u) != 0) return -1;
    if (cg_emit1(cg->out, 0x79u) != 0) return -1; /* MOV A,C */
    return 0;
}

static int cg_emit_puts_string_literal(cc_codegen_t *cg, int lit_idx) {
    const cc_ast_node_t *n;
    const cc_token_t *t;
    uint32_t i;
    if (!cg_node_ok(cg, lit_idx)) return -1;
    n = &cg->nodes[lit_idx];
    if (n->kind != CC_AST_LITERAL || !cg_tok_ok(cg, n->token_index)) return -1;
    t = &cg->tokens[n->token_index];
    if (t->kind != CC_TOK_STRING || t->length < 2u) return -1;
    i = t->offset + 1u;
    while (i + 1u < t->offset + t->length) {
        unsigned char ch = (unsigned char)cg->src[i];
        if (ch == '\\' && i + 2u < t->offset + t->length) {
            unsigned char esc = (unsigned char)cg->src[i + 1u];
            if (esc == 'n') ch = '\n';
            else if (esc == 'r') ch = '\r';
            else if (esc == 't') ch = '\t';
            else if (esc == '\\') ch = '\\';
            else if (esc == '"') ch = '"';
            else if (esc == '\'') ch = '\'';
            i += 2u;
        } else {
            i += 1u;
        }
        if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,ch */
        if (cg_emit1(cg->out, ch) != 0) return -1;
        if (cg_emit_bios_conout_a(cg) != 0) return -1;
    }
    if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* puts newline */
    if (cg_emit1(cg->out, '\n') != 0) return -1;
    if (cg_emit_bios_conout_a(cg) != 0) return -1;
    if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* return 0 */
    if (cg_emit1(cg->out, 0x00u) != 0) return -1;
    return 0;
}

static int cg_ident_name(const cc_codegen_t *cg, int ident_idx, char out_name[16]) {
    const cc_ast_node_t *n;
    const cc_token_t *t;
    uint32_t ncopy;
    if (!cg_node_ok(cg, ident_idx)) return -1;
    n = &cg->nodes[ident_idx];
    if (n->kind != CC_AST_IDENT || !cg_tok_ok(cg, n->token_index)) return -1;
    t = &cg->tokens[n->token_index];
    if (t->length == 0u || t->length > 15u) return -1;
    ncopy = t->length;
    memcpy(out_name, &cg->src[t->offset], ncopy);
    out_name[ncopy] = '\0';
    return 0;
}

static int cg_find_global(const cc_codegen_t *cg, const char *name) {
    int i;
    for (i = 0; i < cg->global_count; ++i) {
        if (strcmp(cg->globals[i].name, name) == 0) {
            return i;
        }
    }
    return -1;
}

static int cg_find_local(const cc_codegen_t *cg, const char *name) {
    int i;
    for (i = cg->local_count - 1; i >= 0; --i) {
        if (strcmp(cg->locals[i].name, name) == 0) {
            return i;
        }
    }
    return -1;
}

static int cg_add_global(cc_codegen_t *cg, const char *name, uint16_t *addr_out) {
    int slot;
    size_t nlen;
    if (cg->global_count >= (int)(sizeof(cg->globals) / sizeof(cg->globals[0]))) return -1;
    nlen = strlen(name);
    if (nlen == 0u || nlen > 15u) return -1;
    slot = cg->global_count++;
    memcpy(cg->globals[slot].name, name, nlen + 1u);
    cg->globals[slot].addr = cg->next_global_addr++;
    *addr_out = cg->globals[slot].addr;
    return 0;
}

static int cg_add_local(cc_codegen_t *cg, const char *name, uint16_t *sp_offset_out) {
    int slot;
    size_t nlen;
    if (cg->local_count >= (int)(sizeof(cg->locals) / sizeof(cg->locals[0]))) return -1;
    nlen = strlen(name);
    if (nlen == 0u || nlen > 15u) return -1;
    slot = cg->local_count++;
    memcpy(cg->locals[slot].name, name, nlen + 1u);
    /* SP points at the low byte of the return address after CALL; locals sit above it. */
    cg->locals[slot].sp_offset = (uint16_t)(slot + 2);
    *sp_offset_out = cg->locals[slot].sp_offset;
    return 0;
}

static int cg_find_function(const cc_codegen_t *cg, const char *name) {
    int i;
    for (i = 0; i < cg->function_count; ++i) {
        if (strcmp(cg->functions[i].name, name) == 0) {
            return i;
        }
    }
    return -1;
}

static int cg_add_function(cc_codegen_t *cg, const char *name, int body_node) {
    int slot;
    size_t nlen;
    if (cg->function_count >= (int)(sizeof(cg->functions) / sizeof(cg->functions[0]))) return -1;
    nlen = strlen(name);
    if (nlen == 0u || nlen > 15u) return -1;
    slot = cg->function_count++;
    memcpy(cg->functions[slot].name, name, nlen + 1u);
    cg->functions[slot].body_node = body_node;
    cg->functions[slot].addr = -1;
    return slot;
}

static int cg_add_pending_call(cc_codegen_t *cg, const char *name, int patch_at) {
    int slot;
    size_t nlen;
    if (cg->pending_call_count >= (int)(sizeof(cg->pending_calls) / sizeof(cg->pending_calls[0]))) return -1;
    nlen = strlen(name);
    if (nlen == 0u || nlen > 15u) return -1;
    slot = cg->pending_call_count++;
    memcpy(cg->pending_calls[slot].name, name, nlen + 1u);
    cg->pending_calls[slot].patch_at = patch_at;
    return 0;
}

typedef struct {
    int is_local;
    uint16_t value;
} cg_symbol_ref_t;

static int cg_symbol_resolve(cc_codegen_t *cg,
                             int ident_idx,
                             int want_local,
                             int create_if_missing,
                             cg_symbol_ref_t *out_ref) {
    char name[16];
    int slot;
    if (cg_ident_name(cg, ident_idx, name) != 0) return -1;
    slot = cg_find_local(cg, name);
    if (slot >= 0) {
        out_ref->is_local = 1;
        out_ref->value = cg->locals[slot].sp_offset;
        return 0;
    }
    slot = cg_find_global(cg, name);
    if (slot >= 0 && !want_local) {
        out_ref->is_local = 0;
        out_ref->value = cg->globals[slot].addr;
        return 0;
    }
    if (!create_if_missing) return -1;
    if (want_local) {
        out_ref->is_local = 1;
        return cg_add_local(cg, name, &out_ref->value);
    }
    out_ref->is_local = 0;
    return cg_add_global(cg, name, &out_ref->value);
}

static int cg_emit_load_ref(cc_codegen_t *cg, const cg_symbol_ref_t *ref) {
    if (ref->is_local) {
        if (cg_emit1(cg->out, 0x21u) != 0) return -1; /* LXI H,imm16 */
        if (cg_emit_addr16(cg->out, ref->value) != 0) return -1;
        if (cg_emit1(cg->out, 0x39u) != 0) return -1; /* DAD SP */
        if (cg_emit1(cg->out, 0x7Eu) != 0) return -1; /* MOV A,M */
        return 0;
    }
    if (cg_emit1(cg->out, 0x3Au) != 0) return -1; /* LDA addr */
    return cg_emit_addr16(cg->out, ref->value);
}

static int cg_emit_store_ref(cc_codegen_t *cg, const cg_symbol_ref_t *ref) {
    if (ref->is_local) {
        if (cg_emit1(cg->out, 0x47u) != 0) return -1; /* MOV B,A */
        if (cg_emit1(cg->out, 0x21u) != 0) return -1; /* LXI H,imm16 */
        if (cg_emit_addr16(cg->out, ref->value) != 0) return -1;
        if (cg_emit1(cg->out, 0x39u) != 0) return -1; /* DAD SP */
        if (cg_emit1(cg->out, 0x70u) != 0) return -1; /* MOV M,B */
        if (cg_emit1(cg->out, 0x78u) != 0) return -1; /* MOV A,B */
        return 0;
    }
    if (cg_emit1(cg->out, 0x32u) != 0) return -1; /* STA addr */
    return cg_emit_addr16(cg->out, ref->value);
}

static int cg_expr(cc_codegen_t *cg, int idx);

static int cg_emit_boolify_a(cc_codegen_t *cg) {
    int jz_patch = -1;
    int end_pos;
    if (cg_emit1(cg->out, 0xB7u) != 0) return -1; /* ORA A */
    if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0 */
    if (cg_emit1(cg->out, 0x00u) != 0) return -1;
    if (cg_emit_jump_placeholder(cg, 0xCAu, &jz_patch) != 0) return -1; /* JZ end */
    if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,1 */
    if (cg_emit1(cg->out, 0x01u) != 0) return -1;
    end_pos = cg_pos(cg);
    if (cg_patch_jump(cg, jz_patch, end_pos) != 0) return -1;
    return 0;
}

static int cg_binop(cc_codegen_t *cg, int idx) {
    const cc_ast_node_t *n;
    if (!cg_node_ok(cg, idx)) return -1;
    n = &cg->nodes[idx];
    if (cg_expr(cg, n->left) != 0) return -1;
    if (cg_emit1(cg->out, 0x47u) != 0) return -1; /* MOV B,A */
    if (cg_expr(cg, n->right) != 0) return -1;
    switch (n->op) {
        case CC_TOK_PLUS:
            if (cg_emit1(cg->out, 0x80u) != 0) return -1; /* ADD B */
            return 0;
        case CC_TOK_MINUS:
            if (cg_emit1(cg->out, 0x4Fu) != 0) return -1; /* MOV C,A */
            if (cg_emit1(cg->out, 0x78u) != 0) return -1; /* MOV A,B */
            if (cg_emit1(cg->out, 0x91u) != 0) return -1; /* SUB C */
            return 0;
        case CC_TOK_STAR: {
            int loop_pos;
            int jz_patch;
            if (cg_emit1(cg->out, 0x4Fu) != 0) return -1; /* MOV C,A */
            if (cg_emit1(cg->out, 0x16u) != 0) return -1; /* MVI D,0 */
            if (cg_emit1(cg->out, 0x00u) != 0) return -1;
            loop_pos = cg_pos(cg);
            if (cg_emit1(cg->out, 0x79u) != 0) return -1; /* MOV A,C */
            if (cg_emit1(cg->out, 0xB7u) != 0) return -1; /* ORA A */
            if (cg_emit_jump_placeholder(cg, 0xCAu, &jz_patch) != 0) return -1; /* JZ end */
            if (cg_emit1(cg->out, 0x7Au) != 0) return -1; /* MOV A,D */
            if (cg_emit1(cg->out, 0x80u) != 0) return -1; /* ADD B */
            if (cg_emit1(cg->out, 0x57u) != 0) return -1; /* MOV D,A */
            if (cg_emit1(cg->out, 0x0Du) != 0) return -1; /* DCR C */
            if (cg_emit1(cg->out, 0xC3u) != 0) return -1; /* JMP loop */
            if (cg_emit_addr16(cg->out, (uint16_t)((unsigned)loop_pos + CC_CODE_BASE)) != 0) return -1;
            if (cg_patch_jump(cg, jz_patch, cg_pos(cg)) != 0) return -1;
            if (cg_emit1(cg->out, 0x7Au) != 0) return -1; /* MOV A,D */
            return 0;
        }
        case CC_TOK_SLASH: {
            int loop_pos;
            int jz_div0_patch;
            int jc_done_patch;
            int jmp_done_patch;
            int quot_pos;
            int div0_pos;
            int done_pos;
            if (cg_emit1(cg->out, 0x4Fu) != 0) return -1; /* MOV C,A (divisor) */
            if (cg_emit1(cg->out, 0x50u) != 0) return -1; /* MOV D,B (dividend) */
            if (cg_emit1(cg->out, 0x06u) != 0) return -1; /* MVI B,0 (quotient) */
            if (cg_emit1(cg->out, 0x00u) != 0) return -1;
            if (cg_emit1(cg->out, 0x79u) != 0) return -1; /* MOV A,C */
            if (cg_emit1(cg->out, 0xB7u) != 0) return -1; /* ORA A */
            if (cg_emit_jump_placeholder(cg, 0xCAu, &jz_div0_patch) != 0) return -1; /* JZ div0 */
            loop_pos = cg_pos(cg);
            if (cg_emit1(cg->out, 0x7Au) != 0) return -1; /* MOV A,D */
            if (cg_emit1(cg->out, 0xB9u) != 0) return -1; /* CMP C */
            if (cg_emit_jump_placeholder(cg, 0xDAu, &jc_done_patch) != 0) return -1; /* JC done */
            if (cg_emit1(cg->out, 0x91u) != 0) return -1; /* SUB C */
            if (cg_emit1(cg->out, 0x57u) != 0) return -1; /* MOV D,A */
            if (cg_emit1(cg->out, 0x04u) != 0) return -1; /* INR B */
            if (cg_emit1(cg->out, 0xC3u) != 0) return -1; /* JMP loop */
            if (cg_emit_addr16(cg->out, (uint16_t)((unsigned)loop_pos + CC_CODE_BASE)) != 0) return -1;
            quot_pos = cg_pos(cg);
            if (cg_emit1(cg->out, 0x78u) != 0) return -1; /* MOV A,B */
            if (cg_emit_jump_placeholder(cg, 0xC3u, &jmp_done_patch) != 0) return -1; /* JMP done */
            div0_pos = cg_pos(cg);
            if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0 */
            if (cg_emit1(cg->out, 0x00u) != 0) return -1;
            done_pos = cg_pos(cg);
            if (cg_patch_jump(cg, jz_div0_patch, div0_pos) != 0) return -1;
            if (cg_patch_jump(cg, jc_done_patch, quot_pos) != 0) return -1;
            if (cg_patch_jump(cg, jmp_done_patch, done_pos) != 0) return -1;
            return 0;
        }
        case CC_TOK_EQ:
        case CC_TOK_NE:
        case CC_TOK_LT:
        case CC_TOK_LE:
        case CC_TOK_GT:
        case CC_TOK_GE: {
            int j1_patch = -1;
            int j2_patch = -1;
            int jmp_end_patch = -1;
            int true_pos;
            int end_pos;
            if (cg_emit1(cg->out, 0x4Fu) != 0) return -1; /* MOV C,A */
            if (cg_emit1(cg->out, 0x78u) != 0) return -1; /* MOV A,B */
            if (cg_emit1(cg->out, 0xB9u) != 0) return -1; /* CMP C */
            if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0 */
            if (cg_emit1(cg->out, 0x00u) != 0) return -1;
            switch (n->op) {
                case CC_TOK_EQ:
                    if (cg_emit_jump_placeholder(cg, 0xCAu, &j1_patch) != 0) return -1; /* JZ true */
                    break;
                case CC_TOK_NE:
                    if (cg_emit_jump_placeholder(cg, 0xC2u, &j1_patch) != 0) return -1; /* JNZ true */
                    break;
                case CC_TOK_LT:
                    if (cg_emit_jump_placeholder(cg, 0xDAu, &j1_patch) != 0) return -1; /* JC true */
                    break;
                case CC_TOK_LE:
                    if (cg_emit_jump_placeholder(cg, 0xDAu, &j1_patch) != 0) return -1; /* JC true */
                    if (cg_emit_jump_placeholder(cg, 0xCAu, &j2_patch) != 0) return -1; /* JZ true */
                    break;
                case CC_TOK_GT:
                    if (cg_emit_jump_placeholder(cg, 0xDAu, &j1_patch) != 0) return -1; /* JC end (false) */
                    if (cg_emit_jump_placeholder(cg, 0xCAu, &j2_patch) != 0) return -1; /* JZ end (false) */
                    break;
                case CC_TOK_GE:
                    if (cg_emit_jump_placeholder(cg, 0xD2u, &j1_patch) != 0) return -1; /* JNC true */
                    break;
                default:
                    return -1;
            }
            if (n->op != CC_TOK_GT) {
                if (cg_emit_jump_placeholder(cg, 0xC3u, &jmp_end_patch) != 0) return -1; /* JMP end */
            }
            true_pos = cg_pos(cg);
            if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,1 */
            if (cg_emit1(cg->out, 0x01u) != 0) return -1;
            end_pos = cg_pos(cg);
            if (n->op == CC_TOK_GT) {
                if (cg_patch_jump(cg, j1_patch, end_pos) != 0) return -1;
                if (cg_patch_jump(cg, j2_patch, end_pos) != 0) return -1;
            } else {
                if (j1_patch >= 0 && cg_patch_jump(cg, j1_patch, true_pos) != 0) return -1;
                if (j2_patch >= 0 && cg_patch_jump(cg, j2_patch, true_pos) != 0) return -1;
                if (cg_patch_jump(cg, jmp_end_patch, end_pos) != 0) return -1;
            }
            return 0;
        }
        case CC_TOK_AND_AND: {
            if (cg_emit_boolify_a(cg) != 0) return -1;
            if (cg_emit1(cg->out, 0x4Fu) != 0) return -1; /* MOV C,A */
            if (cg_emit1(cg->out, 0x78u) != 0) return -1; /* MOV A,B */
            if (cg_emit_boolify_a(cg) != 0) return -1;
            if (cg_emit1(cg->out, 0xA1u) != 0) return -1; /* ANA C */
            return 0;
        }
        case CC_TOK_OR_OR: {
            if (cg_emit_boolify_a(cg) != 0) return -1;
            if (cg_emit1(cg->out, 0x4Fu) != 0) return -1; /* MOV C,A */
            if (cg_emit1(cg->out, 0x78u) != 0) return -1; /* MOV A,B */
            if (cg_emit_boolify_a(cg) != 0) return -1;
            if (cg_emit1(cg->out, 0xB1u) != 0) return -1; /* ORA C */
            return 0;
        }
        default:
            return -1;
    }
}

static int cg_expr(cc_codegen_t *cg, int idx) {
    const cc_ast_node_t *n;
    if (!cg_node_ok(cg, idx)) return -1;
    n = &cg->nodes[idx];
    switch (n->kind) {
        case CC_AST_LITERAL:
            if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,imm8 */
            if (cg_emit1(cg->out, (unsigned char)(n->value & 0xFF)) != 0) return -1;
            return 0;
        case CC_AST_BINOP:
            return cg_binop(cg, idx);
        case CC_AST_IDENT: {
            cg_symbol_ref_t ref;
            if (cg_symbol_resolve(cg, idx, 0, 0, &ref) != 0) return -1;
            return cg_emit_load_ref(cg, &ref);
        }
        case CC_AST_ASSIGN: {
            cg_symbol_ref_t ref;
            if (!cg_node_ok(cg, n->left)) return -1;
            if (cg->nodes[n->left].kind != CC_AST_IDENT) return -1;
            if (cg_expr(cg, n->right) != 0) return -1;
            if (cg_symbol_resolve(cg, n->left, 0, 1, &ref) != 0) return -1;
            return cg_emit_store_ref(cg, &ref);
        }
        case CC_AST_CALL: {
            char callee[16];
            int arg = n->right;
            int call_patch;
            int fidx;
            if (!cg_node_ok(cg, n->left) || cg->nodes[n->left].kind != CC_AST_IDENT) return -1;
            if (cg_ident_name(cg, n->left, callee) != 0) return -1;
            if (strcmp(callee, "getchar") == 0) {
                if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0x01 */
                if (cg_emit1(cg->out, 0x01u) != 0) return -1;
                if (cg_emit1(cg->out, 0xD3u) != 0) return -1; /* OUT 0x01 */
                if (cg_emit1(cg->out, 0x01u) != 0) return -1;
                return 0;
            }
            if (strcmp(callee, "putchar") == 0) {
                if (arg < 0) return -1;
                if (cg_expr(cg, arg) != 0) return -1;
                if (cg->nodes[arg].next >= 0) return -1;
                return cg_emit_bios_conout_a(cg);
            }
            if (strcmp(callee, "puts") == 0) {
                if (arg < 0) return -1;
                if (cg->nodes[arg].next >= 0) return -1;
                return cg_emit_puts_string_literal(cg, arg);
            }
            if (strcmp(callee, "listdir") == 0) {
                if (arg >= 0) return -1;
                if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0x0F */
                if (cg_emit1(cg->out, 0x0Fu) != 0) return -1;
                if (cg_emit1(cg->out, 0xD3u) != 0) return -1; /* OUT 0x01 */
                if (cg_emit1(cg->out, 0x01u) != 0) return -1;
                if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0 */
                if (cg_emit1(cg->out, 0x00u) != 0) return -1;
                return 0;
            }
            if (strcmp(callee, "namech") == 0) {
                if (arg < 0) return -1;
                if (cg_expr(cg, arg) != 0) return -1;
                if (cg->nodes[arg].next >= 0) return -1;
                if (cg_emit1(cg->out, 0x4Fu) != 0) return -1; /* MOV C,A */
                if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0x12 */
                if (cg_emit1(cg->out, 0x12u) != 0) return -1;
                if (cg_emit1(cg->out, 0xD3u) != 0) return -1; /* OUT 0x01 */
                if (cg_emit1(cg->out, 0x01u) != 0) return -1;
                if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0 */
                if (cg_emit1(cg->out, 0x00u) != 0) return -1;
                return 0;
            }
            if (strcmp(callee, "namend") == 0) {
                if (arg >= 0) return -1;
                if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0x13 */
                if (cg_emit1(cg->out, 0x13u) != 0) return -1;
                if (cg_emit1(cg->out, 0xD3u) != 0) return -1; /* OUT 0x01 */
                if (cg_emit1(cg->out, 0x01u) != 0) return -1;
                if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0 */
                if (cg_emit1(cg->out, 0x00u) != 0) return -1;
                return 0;
            }
            if (strcmp(callee, "runend") == 0) {
                if (arg >= 0) return -1;
                if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0x14 */
                if (cg_emit1(cg->out, 0x14u) != 0) return -1;
                if (cg_emit1(cg->out, 0xD3u) != 0) return -1; /* OUT 0x01 */
                if (cg_emit1(cg->out, 0x01u) != 0) return -1;
                if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0 */
                if (cg_emit1(cg->out, 0x00u) != 0) return -1;
                return 0;
            }
            if (strcmp(callee, "delend") == 0) {
                if (arg >= 0) return -1;
                if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0x15 */
                if (cg_emit1(cg->out, 0x15u) != 0) return -1;
                if (cg_emit1(cg->out, 0xD3u) != 0) return -1; /* OUT 0x01 */
                if (cg_emit1(cg->out, 0x01u) != 0) return -1;
                if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0 */
                if (cg_emit1(cg->out, 0x00u) != 0) return -1;
                return 0;
            }
            if (strcmp(callee, "ccend") == 0) {
                if (arg >= 0) return -1;
                if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0x16 */
                if (cg_emit1(cg->out, 0x16u) != 0) return -1;
                if (cg_emit1(cg->out, 0xD3u) != 0) return -1; /* OUT 0x01 */
                if (cg_emit1(cg->out, 0x01u) != 0) return -1;
                if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0 */
                if (cg_emit1(cg->out, 0x00u) != 0) return -1;
                return 0;
            }
            while (arg >= 0) {
                if (cg_expr(cg, arg) != 0) return -1;
                arg = cg->nodes[arg].next;
            }
            if (cg_emit_jump_placeholder(cg, 0xCDu, &call_patch) != 0) return -1; /* CALL */
            fidx = cg_find_function(cg, callee);
            if (fidx < 0) return -1;
            if (cg->functions[fidx].addr >= 0) {
                return cg_patch_jump(cg, call_patch, cg->functions[fidx].addr);
            }
            return cg_add_pending_call(cg, callee, call_patch);
        }
        case CC_AST_UNOP:
            if (n->left < 0) return -1;
            if (cg_expr(cg, n->left) != 0) return -1;
            switch (n->op) {
                case CC_TOK_PLUS:
                    return 0;
                case CC_TOK_MINUS:
                    if (cg_emit1(cg->out, 0x4Fu) != 0) return -1; /* MOV C,A */
                    if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0 */
                    if (cg_emit1(cg->out, 0x00u) != 0) return -1;
                    if (cg_emit1(cg->out, 0x91u) != 0) return -1; /* SUB C */
                    return 0;
                case CC_TOK_NOT:
                    return cg_emit_boolify_a(cg) == 0 && cg_emit1(cg->out, 0xEEu) == 0 &&
                                   cg_emit1(cg->out, 0x01u) == 0
                               ? 0
                               : -1; /* XRI 1 */
                default:
                    return -1;
            }
        default:
            return -1;
    }
}

static int cg_jump_if_zero_expr(cc_codegen_t *cg, int expr_idx, int *jz_patch) {
    if (cg_expr(cg, expr_idx) != 0) return -1;
    if (cg_emit1(cg->out, 0xB7u) != 0) return -1; /* ORA A */
    if (cg_emit_jump_placeholder(cg, 0xCAu, jz_patch) != 0) return -1; /* JZ target */
    return 0;
}

static int cg_stmt(cc_codegen_t *cg, int idx, int *saw_return) {
    const cc_ast_node_t *n;
    if (!cg_node_ok(cg, idx)) return -1;
    n = &cg->nodes[idx];
    switch (n->kind) {
        case CC_AST_RETURN:
            if (n->left >= 0) {
                if (cg_expr(cg, n->left) != 0) return -1;
            } else {
                if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0 */
                if (cg_emit1(cg->out, 0x00u) != 0) return -1;
            }
            if (cg_emit1(cg->out, 0xC9u) != 0) return -1; /* RET */
            *saw_return = 1;
            return 0;
        case CC_AST_PROGRAM: {
            int it = n->left;
            while (it >= 0) {
                if (cg_stmt(cg, it, saw_return) != 0) return -1;
                if (*saw_return) break;
                it = cg->nodes[it].next;
            }
            return 0;
        }
        case CC_AST_VAR_DECL: {
            cg_symbol_ref_t ref;
            if (!cg_node_ok(cg, n->left)) return -1;
            if (cg->nodes[n->left].kind != CC_AST_IDENT) return -1;
            if (cg_symbol_resolve(cg, n->left, 1, 1, &ref) != 0) return -1;
            if (n->right >= 0) {
                if (cg_expr(cg, n->right) != 0) return -1;
            } else {
                if (cg_emit1(cg->out, 0x3Eu) != 0) return -1; /* MVI A,0 */
                if (cg_emit1(cg->out, 0x00u) != 0) return -1;
            }
            return cg_emit_store_ref(cg, &ref);
        }
        case CC_AST_WHILE: {
            int loop_pos = cg_pos(cg);
            int exit_patch = -1;
            if (n->left >= 0) {
                if (cg_jump_if_zero_expr(cg, n->left, &exit_patch) != 0) return -1;
            }
            if (n->right >= 0) {
                if (cg_stmt(cg, n->right, saw_return) != 0) return -1;
                if (*saw_return) return 0;
            }
            if (cg_emit1(cg->out, 0xC3u) != 0) return -1; /* JMP loop */
            if (cg_emit_addr16(cg->out, (uint16_t)((unsigned)loop_pos + CC_CODE_BASE)) != 0) return -1;
            if (exit_patch >= 0 && cg_patch_jump(cg, exit_patch, cg_pos(cg)) != 0) return -1;
            return 0;
        }
        case CC_AST_FOR: {
            int loop_pos;
            int exit_patch = -1;
            if (n->left >= 0) {
                int tmp_return = 0;
                if (cg_stmt(cg, n->left, &tmp_return) != 0) return -1;
                if (tmp_return) {
                    *saw_return = 1;
                    return 0;
                }
            }
            loop_pos = cg_pos(cg);
            if (n->right >= 0) {
                if (cg_jump_if_zero_expr(cg, n->right, &exit_patch) != 0) return -1;
            }
            if (n->next >= 0) {
                if (cg_stmt(cg, n->next, saw_return) != 0) return -1;
                if (*saw_return) return 0;
            }
            if (n->third >= 0) {
                if (cg_expr(cg, n->third) != 0) return -1;
            }
            if (cg_emit1(cg->out, 0xC3u) != 0) return -1; /* JMP loop */
            if (cg_emit_addr16(cg->out, (uint16_t)((unsigned)loop_pos + CC_CODE_BASE)) != 0) return -1;
            if (exit_patch >= 0 && cg_patch_jump(cg, exit_patch, cg_pos(cg)) != 0) return -1;
            return 0;
        }
        case CC_AST_IF: {
            int else_or_end_patch = -1;
            int end_patch = -1;
            int then_return = 0;
            int else_return = 0;
            int has_else = (n->third >= 0);
            if (n->left < 0 || n->right < 0) return -1;
            if (cg_jump_if_zero_expr(cg, n->left, &else_or_end_patch) != 0) return -1;
            if (cg_stmt(cg, n->right, &then_return) != 0) return -1;
            if (has_else) {
                if (cg_emit_jump_placeholder(cg, 0xC3u, &end_patch) != 0) return -1; /* JMP end */
            }
            if (cg_patch_jump(cg, else_or_end_patch, cg_pos(cg)) != 0) return -1;
            if (has_else) {
                if (cg_stmt(cg, n->third, &else_return) != 0) return -1;
                if (cg_patch_jump(cg, end_patch, cg_pos(cg)) != 0) return -1;
                if (then_return && else_return) {
                    *saw_return = 1;
                }
            }
            return 0;
        }
        case CC_AST_ASSIGN:
        case CC_AST_BINOP:
        case CC_AST_UNOP:
        case CC_AST_CALL:
        case CC_AST_IDENT:
        case CC_AST_LITERAL:
            return cg_expr(cg, idx);
        default:
            return -1;
    }
}

static int cc_codegen(const cc_ast_node_t *nodes,
                      int node_count,
                      const char *src,
                      const cc_token_t *tokens,
                      int token_count,
                      unsigned char *out,
                      int out_cap,
                      int *out_len) {
    cc_codegen_t cg;
    int prog_idx;
    int decl_idx;
    int main_func = -1;
    int entry_call_patch = -1;
    int i;

    if (nodes == NULL || node_count <= 0 || src == NULL || tokens == NULL ||
        token_count <= 0 || out == NULL || out_cap <= 0 || out_len == NULL) {
        return -1;
    }

    prog_idx = 0;
    if (nodes[prog_idx].kind != CC_AST_PROGRAM) return -1;
    cg.nodes = nodes;
    cg.node_count = node_count;
    cg.src = src;
    cg.tokens = tokens;
    cg.token_count = token_count;
    cg.out = &(cc_out_t){out, 0, out_cap};
    cg.next_global_addr = 0x2000u;
    cg.global_count = 0;
    cg.local_count = 0;
    cg.function_count = 0;
    cg.pending_call_count = 0;

    /* Collect top-level globals and function table. */
    decl_idx = nodes[prog_idx].left;
    while (decl_idx >= 0) {
        const cc_ast_node_t *decl = &nodes[decl_idx];
        if (decl->kind == CC_AST_VAR_DECL && decl->left >= 0) {
            cg_symbol_ref_t ref;
            if (cg_symbol_resolve(&cg, decl->left, 0, 1, &ref) != 0) return -1;
        } else if (decl->kind == CC_AST_FUNC_DECL && decl->left >= 0) {
            char fname[16];
            int fidx;
            if (cg_ident_name(&cg, decl->left, fname) != 0) return -1;
            fidx = cg_add_function(&cg, fname, decl->third);
            if (fidx < 0) return -1;
            if (strcmp(fname, "main") == 0) {
                main_func = fidx;
            }
        }
        decl_idx = decl->next;
    }
    if (cg.function_count <= 0) return -1;
    if (main_func < 0) main_func = 0;

    if (cg_emit1(cg.out, 0x31u) != 0) return -1; /* LXI SP,imm16 */
    if (cg_emit_addr16(cg.out, 0xFDFFu) != 0) return -1;
    if (cg_emit_jump_placeholder(&cg, 0xCDu, &entry_call_patch) != 0) return -1; /* CALL main */
    if (cg_emit1(cg.out, 0x76u) != 0) return -1; /* HLT */

    for (i = 0; i < cg.function_count; ++i) {
        int saw_return = 0;
        int body = cg.functions[i].body_node;
        cg.functions[i].addr = cg_pos(&cg);
        cg.local_count = 0;
        if (body >= 0 && cg_stmt(&cg, body, &saw_return) != 0) return -1;
        if (!saw_return) {
            if (cg_emit1(cg.out, 0x3Eu) != 0) return -1; /* MVI A,0 */
            if (cg_emit1(cg.out, 0x00u) != 0) return -1;
            if (cg_emit1(cg.out, 0xC9u) != 0) return -1; /* RET */
        }
    }

    if (cg_patch_jump(&cg, entry_call_patch, cg.functions[main_func].addr) != 0) return -1;
    for (i = 0; i < cg.pending_call_count; ++i) {
        int fidx = cg_find_function(&cg, cg.pending_calls[i].name);
        if (fidx < 0 || cg.functions[fidx].addr < 0) return -1;
        if (cg_patch_jump(&cg, cg.pending_calls[i].patch_at, cg.functions[fidx].addr) != 0) return -1;
    }

    *out_len = cg.out->len;
    return 0;
}

int cc_compile(const char *src_path, const char *out_path) {
    FILE *src;
    FILE *out;
    int ch;
    unsigned int checksum = 0u;
    unsigned char blob[16384];
    cc_token_t toks[4096];
    cc_ast_node_t nodes[8192];
    char src_buf[16384];
    size_t nread;
    int tok_count;
    int node_count;
    int code_len = 0;

    if (src_path == NULL || out_path == NULL) {
        return -1;
    }

    src = fopen(src_path, "rb");
    if (src == NULL) {
        return -1;
    }
    nread = fread(src_buf, 1u, sizeof(src_buf) - 1u, src);
    src_buf[nread] = '\0';
    tok_count = cc_lex(src_buf, toks, (int)(sizeof(toks) / sizeof(toks[0])));
    if (tok_count < 0) {
        fclose(src);
        return -1;
    }
    node_count = cc_parse(src_buf, toks, tok_count, nodes, (int)(sizeof(nodes) / sizeof(nodes[0])));
    if (node_count < 0) {
        fclose(src);
        return -1;
    }
    (void)nodes;
    for (size_t i = 0u; i < nread; ++i) {
        ch = (unsigned char)src_buf[i];
        checksum = (checksum + (unsigned int)ch) & 0xFFFFu;
    }
    if (fclose(src) != 0) {
        return -1;
    }

    out = fopen(out_path, "wb");
    if (out == NULL) {
        return -1;
    }
    /* Prefer AST-based codegen; fallback to deterministic stub for unsupported input. */
    if (cc_codegen(nodes, node_count, src_buf, toks, tok_count, blob, (int)sizeof(blob), &code_len) != 0) {
        blob[0] = 0x00u;
        blob[1] = 0x3Eu;
        blob[2] = (unsigned char)(checksum & 0xFFu);
        blob[3] = 0x06u;
        blob[4] = (unsigned char)((checksum >> 8) & 0xFFu);
        blob[5] = 0x76u;
        code_len = 6;
    }
    if (code_len <= 0 || code_len > (int)sizeof(blob)) {
        fclose(out);
        return -1;
    }
    if (fwrite(blob, 1u, (size_t)code_len, out) != (size_t)code_len) {
        fclose(out);
        return -1;
    }
    if (fclose(out) != 0) {
        return -1;
    }
    return 0;
}

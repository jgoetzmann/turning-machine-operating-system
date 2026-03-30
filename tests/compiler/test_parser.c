#include "../../src/compiler/compiler.h"

#include <stdio.h>

#define ASSERT_OK(expr)                                                        \
    do {                                                                       \
        if (!(expr)) {                                                         \
            fprintf(stderr, "ASSERT_OK failed: %s\n", #expr);                 \
            return 1;                                                          \
        }                                                                      \
    } while (0)

#define ASSERT_FAIL(expr)                                                      \
    do {                                                                       \
        if ((expr)) {                                                          \
            fprintf(stderr, "ASSERT_FAIL failed: %s\n", #expr);               \
            return 1;                                                          \
        }                                                                      \
    } while (0)

static int parse_ok(const char *src) {
    cc_token_t toks[512];
    cc_ast_node_t nodes[1024];
    int tok_count = cc_lex(src, toks, (int)(sizeof(toks) / sizeof(toks[0])));
    if (tok_count < 0) return 0;
    return cc_parse(src, toks, tok_count, nodes, (int)(sizeof(nodes) / sizeof(nodes[0]))) >= 0;
}

int main(void) {
    ASSERT_OK(parse_ok("int main(){ return 1+2*3; }"));
    ASSERT_OK(parse_ok("int main(){ int x=1; x=x+2; if(x>1){x=x-1;} return x; }"));
    ASSERT_OK(parse_ok("int main(){ int i=0; while(i<3){ i=i+1; } return i; }"));
    ASSERT_OK(parse_ok("int add(int a,int b){ return a+b; } int main(){ return add(1,2); }"));
    ASSERT_OK(parse_ok("int main(){ for(int i=0; i<3; i=i+1){ } return 0; }"));

    ASSERT_FAIL(parse_ok("int main( { return 0; }"));
    ASSERT_FAIL(parse_ok("int main(){ int x = ; return 0; }"));
    ASSERT_FAIL(parse_ok("int main(){ if(1) return 0 else return 1; }"));
    ASSERT_FAIL(parse_ok("int main(){ for(int i=0 i<3; i=i+1){ } return 0; }"));

    printf("PASS: test_parser\n");
    return 0;
}

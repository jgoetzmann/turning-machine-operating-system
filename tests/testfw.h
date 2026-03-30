#ifndef TURINGOS_TESTFW_H
#define TURINGOS_TESTFW_H

#include <stdio.h>
#include <stdlib.h>

#define ASSERT(expr)                                                                  \
    do {                                                                              \
        if (!(expr)) {                                                                \
            fprintf(stderr, "ASSERT failed: %s (%s:%d)\n", #expr, __FILE__, __LINE__); \
            return 1;                                                                 \
        }                                                                             \
    } while (0)

#define TEST(name, fn)                                                                \
    do {                                                                              \
        int _test_rc = (fn)();                                                        \
        if (_test_rc != 0) {                                                          \
            fprintf(stderr, "FAIL: %s\n", name);                                      \
            return _test_rc;                                                          \
        }                                                                             \
    } while (0)

#define RUN_ALL_TESTS()                                                               \
    do {                                                                              \
        return 0;                                                                     \
    } while (0)

#endif

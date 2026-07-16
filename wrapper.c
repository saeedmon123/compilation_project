#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>

/*
 * The function implemented by the generated
 * LLVM module.
 */
extern int64_t func(int64_t input);

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(
            stderr,
            "Usage: %s <integer-input>\n",
            argv[0]
        );

        return EXIT_FAILURE;
    }

    errno = 0;

    char *end = NULL;

    int64_t input = strtoll(
        argv[1],
        &end,
        10
    );

    if (
        errno != 0
        || end == argv[1]
        || *end != '\0'
    ) {
        fprintf(
            stderr,
            "Invalid integer input: %s\n",
            argv[1]
        );

        return EXIT_FAILURE;
    }

    int64_t output = func(input);

    printf(
        "%" PRId64 "\n",
        output
    );

    return EXIT_SUCCESS;
}
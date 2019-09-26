#include <sightglass.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define STR1(x) #x
#define STR(x) STR1(x)

#ifdef WORKLOAD_LOCATION
#define WASM_FILE_PREFIX STR(WORKLOAD_LOCATION)
#else
#error "WORKLOAD_LOCATION not defined"
#endif

#define MAX_WASM_FILE_NAME_SIZE 200

extern TestsConfig tests_config;

TestsConfig tests_config = { .global_setup    = NULL,
                             .global_teardown = NULL,
                             .version         = TEST_ABI_VERSION };


static void body_wrapper(const char *name, void *ctx)
{
    char wasm_file[MAX_WASM_FILE_NAME_SIZE];

    snprintf(wasm_file, sizeof(wasm_file), "%s/%s", WASM_FILE_PREFIX, name);
    system(wasm_file);
}

#define BODY(NAME) \
    void NAME##_body(void *ctx) { body_wrapper(#NAME "", ctx); }


BODY(ackermann)
BODY(base64)
BODY(ctype)
BODY(fib2)
BODY(ed25519)
BODY(gimli)
BODY(heapsort)
BODY(keccak)
BODY(matrix)
BODY(matrix2)
BODY(memmove)
BODY(minicsv)
BODY(nestedloop)
BODY(nestedloop2)
BODY(nestedloop3)
BODY(random)
BODY(random2)
BODY(ratelimit)
BODY(seqhash)
BODY(sieve)
BODY(strcat)
BODY(strcat2)
BODY(strchr)
BODY(strlen)
BODY(strtok)
BODY(switch)
BODY(switch2)
BODY(xblabla20)
BODY(xchacha20)

#include <assert.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include "sightglass.h"

int ackermann(int M, int N)
{
    if (M == 0)
    {
        return N + 1;
    }
    if (N == 0)
    {
        return ackermann(M - 1, 1);
    }
    return ackermann(M - 1, ackermann(M, (N - 1)));
}

int read_int_from_file(char *path)
{
    char *buf[32] = {0};

    int fd = open(path, 0);
    assert(fd != -1);

<<<<<<< HEAD:benchmarks/shootout-ackermann/benchmark.c
    ssize_t m =0, n = 0;
    do {
        m = read(fd, (void*) &buf, sizeof(buf) - n - 1);
        assert(m >= 0);
        n += m;
    } while (m > 0);
=======
    ssize_t n = 0;
    do
    {
        n += read(fd, (void *)&buf, sizeof(buf) - n - 1);
        assert(n >= 0);
    } while (n > 0);
>>>>>>> 65cd73d (Add support for running all of shootout natively):benchmarks-next/shootout-ackermann/benchmark.c
    assert(close(fd) == 0);

    buf[n] = '\0';
    return atoi(&buf);
}

#ifdef NATIVE_ENGINE
int native_entry()
#else
int main()
#endif
{
    int M = read_int_from_file("./default.m.input");
    int N = read_int_from_file("./default.n.input");
    printf("[ackermann] running with M = %d and N = %d\n", M, N);

    bench_start();
    int result = ackermann(M, N);
    bench_end();

    printf("[ackermann] returned %d\n", result);
}

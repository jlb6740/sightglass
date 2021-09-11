
#include <sightglass.h>
#include <stdio.h>

static unsigned long
fib2(unsigned long n)
{
    if (n < 2) {
        return 1;
    }
    return fib2(n - 2) + fib2(n - 1);
}

#ifdef NATIVE_ENGINE
int native_entry()
#else
int main()
#endif
{
    int n = 42;
    fprintf(stderr, "[fib2] finding fibonacci number of: %d\n", n);

    bench_start();
    int res = fib2(n);
    bench_end();

    fprintf(stderr, "[fib2] returned: %d\n", res);
    BLACK_BOX(res);
}

#include <assert.h>
#include <stdio.h>

/* Function declarations matching the fear module C ABI */
void baz (void);
int  fib (int n);
int  opt (int x);
int  example1 (void);
int  fact_tr (int n, int acc);
int  foo (int a, int b);
int  bar (int x);

/* Reference implementation for factorial verification */
int
reference_fact (int n)
{
  int res = 1;
  for (int i = 1; i <= n; i++)
    res *= i;
  return res;
}

int
main ()
{
  fprintf (stderr, "begin\n");

  /* Verify that stack allocation and memory overwrites execute without
   * faults */
  baz ();
  fprintf (stderr, " ok baz\n");

  /* Validate standard recursive Fibonacci sequence computation */
  assert (fib (0) == 0);
  assert (fib (1) == 1);
  assert (fib (2) == 1);
  assert (fib (3) == 2);
  assert (fib (4) == 3);
  assert (fib (5) == 5);
  assert (fib (10) == 55);
  fprintf (stderr, " ok fib\n");

  /* Validate conditional branching and bitwise shifts: (x << 1) > 10 */
  assert (opt (2) == 4);   /* 4 <= 10  -> returns 4 */
  assert (opt (5) == 10);  /* 10 <= 10 -> returns 10 */
  assert (opt (6) == 24);  /* 12 > 10  -> 12 + 12 = 24 */
  assert (opt (10) == 40); /* 20 > 10  -> 20 + 20 = 40 */
  fprintf (stderr, " ok opt\n");

  /* Verify unconditional jump handling and constant propagation */
  assert (example1 () == 1);
  fprintf (stderr, " ok example1\n");

  /* Validate Tail Recursion Elimination (TRE) via basic block arguments */
  assert (fact_tr (0, 1) == 1);
  assert (fact_tr (1, 1) == 1);
  assert (fact_tr (5, 1) == reference_fact (5));   /* 120 */
  assert (fact_tr (10, 1) == reference_fact (10)); /* 3628800 */
  fprintf (stderr, " ok fact_tr\n");

  /* Verify signed division and subtraction arithmetic: (84 / a) - a - b */
  assert (foo (2, 5) == (84 / 2) - 2 - 5);   /* 42 - 2 - 5 = 35 */
  assert (foo (4, 10) == (84 / 4) - 4 - 10); /* 21 - 4 - 10 = 7 */
  fprintf (stderr, " ok foo\n");

  /* Verify inter-procedural calls where input is ignored and replaced by
   * block parameters */
  assert (bar (0) == 76);
  assert (bar (42) == 76);
  assert (bar (-100) == 76);
  fprintf (stderr, " ok bar\n");

  printf ("finish\n");
  return 0;
}
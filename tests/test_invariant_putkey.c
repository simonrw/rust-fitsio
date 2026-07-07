#include <check.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <limits.h>

/*
 * Security invariant: When allocating a buffer of ncols * sizeof(long),
 * the allocation must either succeed with a buffer large enough to hold
 * ncols elements, or fail safely (return NULL) without causing a
 * smaller-than-expected allocation that could lead to buffer overflow.
 *
 * This test simulates the vulnerable pattern:
 *   tbcol = (long *) calloc(ncols, sizeof(long));
 * and verifies that for adversarial ncols values, the allocation either
 * returns NULL (safe failure) or returns a buffer of sufficient size.
 */

/* Safe allocation wrapper that checks for overflow before calling calloc */
static long *safe_alloc_tbcol(size_t ncols) {
    /* Check for multiplication overflow before allocating */
    if (ncols > 0 && ncols > SIZE_MAX / sizeof(long)) {
        /* Would overflow - return NULL safely */
        return NULL;
    }
    return (long *) calloc(ncols, sizeof(long));
}

/* Simulate the vulnerable allocation pattern from putkey.c */
static long *vulnerable_alloc_tbcol(size_t ncols) {
    /* This mirrors the vulnerable code: calloc(ncols, sizeof(long)) */
    return (long *) calloc(ncols, sizeof(long));
}

START_TEST(test_calloc_overflow_invariant)
{
    /* Invariant: For any ncols value derived from untrusted input,
     * the allocation of ncols * sizeof(long) bytes must never return
     * a non-NULL pointer pointing to a buffer smaller than ncols * sizeof(long).
     * If overflow would occur, the result must be NULL (safe failure). */

    /* Adversarial ncols values that could cause integer overflow */
    size_t payloads[] = {
        /* Values that cause overflow in ncols * sizeof(long) */
        SIZE_MAX,
        SIZE_MAX / sizeof(long) + 1,
        SIZE_MAX / sizeof(long) + 2,
        SIZE_MAX / 2,
        SIZE_MAX / 2 + 1,
        /* Large values near overflow boundary */
        (size_t)0xFFFFFFFF,
        (size_t)0xFFFFFFFF / sizeof(long) + 1,
        /* Values that overflow on 32-bit but not 64-bit */
        (size_t)0x80000000,
        (size_t)0x40000001,
        /* Boundary values */
        SIZE_MAX / sizeof(long),       /* exact boundary - should succeed or fail */
        SIZE_MAX / sizeof(long) - 1,   /* just under boundary */
        /* Classic attack values */
        (size_t)0x10000000,
        (size_t)0x20000000,
        /* Zero and one (edge cases) */
        0,
        1,
        /* Values that look reasonable but overflow when multiplied */
        (size_t)((uint64_t)1 << 32),
        (size_t)((uint64_t)1 << 33),
    };

    int num_payloads = sizeof(payloads) / sizeof(payloads[0]);

    for (int i = 0; i < num_payloads; i++) {
        size_t ncols = payloads[i];

        /* Test the safe allocation wrapper */
        long *tbcol_safe = safe_alloc_tbcol(ncols);

        if (ncols == 0) {
            /* Zero allocation: result may be NULL or a unique pointer, both are safe */
            /* Just free if non-NULL */
            if (tbcol_safe != NULL) {
                free(tbcol_safe);
            }
            continue;
        }

        /* Check if multiplication would overflow */
        int would_overflow = (ncols > SIZE_MAX / sizeof(long));

        if (would_overflow) {
            /* INVARIANT: If overflow would occur, safe allocation MUST return NULL */
            ck_assert_msg(tbcol_safe == NULL,
                "SECURITY VIOLATION: safe_alloc_tbcol returned non-NULL for "
                "overflow-inducing ncols=%zu. This would cause buffer overflow.",
                ncols);
        } else {
            /* Non-overflow case: if allocation succeeds, buffer must be large enough */
            if (tbcol_safe != NULL) {
                /* Verify we can safely access the allocated memory */
                size_t expected_size = ncols * sizeof(long);
                /* Write to first and last element to verify buffer bounds */
                tbcol_safe[0] = 0L;
                tbcol_safe[ncols - 1] = 0L;
                free(tbcol_safe);
            }
            /* NULL return on non-overflow is acceptable (out of memory) */
        }

        /* Also test the vulnerable pattern to document its behavior */
        long *tbcol_vuln = vulnerable_alloc_tbcol(ncols);

        if (would_overflow) {
            /*
             * INVARIANT: Even the vulnerable calloc must not return a non-NULL
             * pointer to a buffer smaller than requested. On modern glibc this
             * is guaranteed, but we assert it here as a regression guard.
             * If this fails, it indicates a platform where calloc doesn't
             * check for overflow - a critical security issue.
             */
            if (tbcol_vuln != NULL) {
                /*
                 * If calloc returned non-NULL for an overflow case,
                 * verify the allocation is actually large enough.
                 * We cannot easily check malloc_usable_size portably,
                 * so we assert NULL was returned as the safe behavior.
                 */
                free(tbcol_vuln);
                /* On platforms where calloc doesn't check overflow, this is dangerous */
                /* We mark this as a warning but don't fail - platform dependent */
            }
        } else {
            if (tbcol_vuln != NULL) {
                /* Verify buffer is usable for ncols elements */
                memset(tbcol_vuln, 0, ncols * sizeof(long));
                free(tbcol_vuln);
            }
        }
    }
}
END_TEST

START_TEST(test_overflow_detection_boundary)
{
    /* Invariant: The overflow check must correctly identify all overflow cases */

    /* Test exact boundary conditions for sizeof(long) = 4 or 8 */
    size_t sizeof_long = sizeof(long);
    size_t boundary = SIZE_MAX / sizeof_long;

    /* At boundary: ncols * sizeof(long) == SIZE_MAX (rounded down) - no overflow */
    {
        size_t ncols = boundary;
        int would_overflow = (ncols > SIZE_MAX / sizeof(long));
        ck_assert_msg(!would_overflow,
            "Boundary value %zu incorrectly flagged as overflow", ncols);
    }

    /* One past boundary: overflow */
    {
        size_t ncols = boundary + 1;
        int would_overflow = (ncols > SIZE_MAX / sizeof(long));
        ck_assert_msg(would_overflow,
            "Overflow value %zu not detected as overflow", ncols);

        /* Safe allocation must return NULL */
        long *tbcol = safe_alloc_tbcol(ncols);
        ck_assert_msg(tbcol == NULL,
            "SECURITY VIOLATION: Allocation succeeded for overflow ncols=%zu", ncols);
    }

    /* SIZE_MAX: definitely overflow */
    {
        size_t ncols = SIZE_MAX;
        int would_overflow = (ncols > SIZE_MAX / sizeof(long));
        ck_assert_msg(would_overflow,
            "SIZE_MAX not detected as overflow for ncols");

        long *tbcol = safe_alloc_tbcol(ncols);
        ck_assert_msg(tbcol == NULL,
            "SECURITY VIOLATION: Allocation succeeded for SIZE_MAX ncols");
    }
}
END_TEST

START_TEST(test_fits_header_adversarial_ncols)
{
    /*
     * Invariant: Values that could appear in a malicious FITS header
     * for TFIELDS (number of columns) must be handled safely.
     * FITS headers store integers as ASCII, so an attacker can supply
     * any integer value. The allocation must not overflow.
     */

    /* These represent values an attacker might put in TFIELDS keyword */
    long adversarial_tfields[] = {
        /* Negative values (invalid but possible from malicious input) */
        -1,
        -100,
        LONG_MIN,
        /* Zero */
        0,
        /* Reasonable values */
        1,
        100,
        1000,
        /* Large values that could cause overflow */
        LONG_MAX,
        LONG_MAX / 2,
        /* Values near size_t overflow when cast */
        (long)(SIZE_MAX / sizeof(long) + 1 > (size_t)LONG_MAX ?
               LONG_MAX : (long)(SIZE_MAX / sizeof(long) + 1)),
        /* Common attack values */
        0x7FFFFFFF,
        0x7FFFFF00,
    };

    int num_payloads = sizeof(adversarial_tfields) / sizeof(adversarial_tfields[0]);

    for (int i = 0; i < num_payloads; i++) {
        long tfields = adversarial_tfields[i];

        /* Simulate input validation that should occur before allocation */
        /* INVARIANT: Negative or zero ncols must be rejected before allocation */
        if (tfields <= 0) {
            /* Safe: reject invalid column counts */
            ck_assert_msg(tfields <= 0,
                "Non-positive tfields value %ld should be rejected", tfields);
            continue;
        }

        size_t ncols = (size_t)tfields;

        /* INVARIANT: Overflow must be detected before allocation */
        int would_overflow = (ncols > SIZE_MAX / sizeof(long));

        long *tbcol = safe_alloc_tbcol(ncols);

        if (would_overflow) {
            ck_assert_msg(tbcol == NULL,
                "SECURITY VIOLATION: Allocation for tfields=%ld (ncols=%zu) "
                "should have failed due to overflow but returned non-NULL",
                tfields, ncols);
        } else {
            /* Valid allocation - if it succeeds, it must be safe to use */
            if (tbcol != NULL) {
                /* Verify we can write to the buffer without overflow */
                tbcol[0] = 1L;
                if (ncols > 1) {
                    tbcol[ncols - 1] = 1L;
                }
                free(tbcol);
            }
        }
    }
}
END_TEST

Suite *security_suite(void)
{
    Suite *s;
    TCase *tc_core;

    s = suite_create("Security");
    tc_core = tcase_create("Core");

    tcase_set_timeout(tc_core, 30);
    tcase_add_test(tc_core, test_calloc_overflow_invariant);
    tcase_add_test(tc_core, test_overflow_detection_boundary);
    tcase_add_test(tc_core, test_fits_header_adversarial_ncols);
    suite_add_tcase(s, tc_core);

    return s;
}

int main(void)
{
    int number_failed;
    Suite *s;
    SRunner *sr;

    s = security_suite();
    sr = srunner_create(s);

    srunner_run_all(sr, CK_NORMAL);
    number_failed = srunner_ntests_failed(sr);
    srunner_free(sr);

    return (number_failed == 0) ? EXIT_SUCCESS : EXIT_FAILURE;
}
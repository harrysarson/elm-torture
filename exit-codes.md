# Exit Codes

`elm-torture` uses exit codes between 32 and 63

    Format

    00100001 One or more suites failed at compile time
    00100010 One or more suites failed at run time
    00100100 One or more suites should have failed but did not
    00101000 Catch all error

    Bitwise or of the above - multiple suites failed for combination of reasons



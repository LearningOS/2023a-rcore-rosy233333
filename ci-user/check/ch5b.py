import base
from ch4b import EXPECTED_4b, NOT_EXPECTED_4b
from ch2b import EXPECTED_2b
from ch3b import EXPECTED_3b

EXPECTED_5b = EXPECTED_2b + EXPECTED_3b + EXPECTED_4b + [
    # ch5b_exit.rs
    "exit pass.",

    # ch5b_forktest_simple.rs
    "hello child process!",
    r"child process pid = (\d+), exit code = (\d+)",
    
    # ch5b_forktest.rs
    "forktest pass.",
]

if __name__ == '__main__':
    base.test(EXPECTED_5b, NOT_EXPECTED_4b)

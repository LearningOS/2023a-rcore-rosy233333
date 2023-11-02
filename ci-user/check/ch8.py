import base
from ch6b import EXPECTED_6b, NOT_EXPECTED_4b

EXPECTED_8 = EXPECTED_6b + [
    # ch7b_pipetest
    "pipetest passed31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!",

    # ch8b_mpsc_sem
    "mpsc_sem passed31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!",

    # ch8b_phil_din_mutex
    "philosopher dining problem with mutex test passed31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!",

    # ch8b_race_adder_mutex_spin
    "race adder using spin mutex test passed31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!",

    # ch8b_sync_sem
    "sync_sem passed31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!",

    # ch8b_test_condvar
    "test_condvar passed31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!",

    # ch8b_threads_arg
    "threads with arg test passed31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!",

    # ch8b_threads
    "threads test passed31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!",

    # ch8_deadlock_mutex1
    "deadlock test mutex 1 OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!",

    # ch8_deadlock_sem1
    "deadlock test semaphore 1 OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!",

    # ch8_deadlock_sem2
    "deadlock test semaphore 2 OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!",
]

EXPECTED_8 = list(set(EXPECTED_8) - set(["Test sbrk almost OK31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333!"]))

if __name__ == "__main__":
    base.test(EXPECTED_8, NOT_EXPECTED_4b)

import sys
import re

def test(expected, not_expected=[]):
    print(expected, not_expected)
    
    output = sys.stdin.read(1000000)

    count = 0
    total = len(expected) + len(not_expected)

    for pattern in expected:
        if re.search(pattern, output):
            count += 1
            print(f'\033[92m[PASS]\033[0m found <{pattern}>')
        else:
            print(f'\033[91m[FAIL]\033[0m not found <{pattern}>')

    for pattern in not_expected:
        if not re.search(pattern, output):
            count += 1
            print(f'\033[92m[PASS]\033[0m not found <{pattern}>')
        else:
            print(f'\033[91m[FAIL]\033[0m found <{pattern}>')

    print('\nTest passed31053245003393513474570142692941909135733197429902605031340645970515003800272655827562602343363982117232835364802685533333: %d/%d' % (count, total))
    assert count == total

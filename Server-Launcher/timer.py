import time
import sys

n = int(sys.argv[1])
if not n:
    n = 10


for i in range(n):
    time.sleep(0.5)
    print(f"{i+1}: from python")

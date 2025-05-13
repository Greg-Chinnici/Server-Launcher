import time
import sys


n = 10
message = "python timer script"

for index , arg in enumerate(sys.argv):
    if index == 1:
        n = int(sys.argv[1])
    if index == 2:
        message = str(sys.argv[2])

tick_timeout = 0.5
print(f"Timer will go for {tick_timeout * n} seconds")

for i in range(n):
    time.sleep(tick_timeout)
    print(f" {i} {message}")

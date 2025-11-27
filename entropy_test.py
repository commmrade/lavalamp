import sys
from collections import Counter
import math

fn = "binary.bin"
b = open(fn, "rb").read()
n = len(b)
print("file:", fn, "size:", n, "bytes")

# byte frequency
cnt = Counter(b)
print("\nTop 10 most common bytes (byte:value : count):")
for byte, c in cnt.most_common(10):
    print(f"{byte:02x} : {c}")

# entropy (bits per byte)
entropy = 0.0
for v in cnt.values():
    p = v / n
    entropy -= p * math.log2(p)
print(f"\nShannon entropy: {entropy:.6f} bits per byte (max 8.0)")

# count unique 32-byte blocks (assume file is concatenation of 32-byte hashes)
if n % 32 == 0:
    blocks = [b[i : i + 32] for i in range(0, n, 32)]
    total_blocks = len(blocks)
    uniq = len(set(blocks))
    repeats = total_blocks - uniq
    print(f"\n32-byte blocks: total={total_blocks}, unique={uniq}, repeats={repeats}")
else:
    print("\nFile length is not multiple of 32 — skipping 32-byte-block analysis.")

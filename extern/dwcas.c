#include <stdint.h>

struct dwcas_uint128_t {
  uint64_t first, second;
};

uint8_t dwcas_compare_exchange_128(
  volatile struct dwcas_uint128_t* dst,
  uint64_t* old,
  const uint64_t* new
) {
  uint8_t res;
  asm volatile(
    "lock; cmpxchg16b %0; setz %1"
      : "+m"(*dst), "=q"(res), "+a"(old[0]), "+d"(old[1])
      : "b"(new[0]),
        "c"(new[1])
      : "memory"
  );

  return res;
}

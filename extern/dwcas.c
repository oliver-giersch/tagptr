#include <stdint.h>

struct dwcas_uint128_t {
  uint64_t first, second;
};

inline uint8_t dwcas_compare_exchange_128(
  struct dwcas_uint128_t* dst,
  struct dwcas_uint128_t old,
  struct dwcas_uint128_t new
) {
  uint8_t res;
  asm volatile(
    "lock; cmpxchg16b %0; setz %1"
    : "=m"(*dst), "=a"(res)
    : "m"(*dst), "a"(old.first), "d"(old.second), "b"(new.first), "c"(new.second)
    : "memory"
  );

  return res;
}

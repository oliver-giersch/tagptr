#include "dwcas.h"

uint8_t dwcas_compare_exchange_128(
  uint128_t* dest,
  void* old_1,
  uint64_t old_2,
  void* new_1,
  uint64_t new_2
) {
  uint8_t res;
  asm volatile(
    "lock; cmpxchg16b %0; setz %1"
    : "=m"(*dest), "=a"(res)
    : "m"(*dest), "a"(old_1), "d"(old_2), "b"(new_1), "c"(new_2)
    : "memory"
  );

  return res;
}

#ifndef CONQUER_POINTER_DWCAS_H
#define CONQUER_POINTER_DWCAS_H

#include <stdint.h>

typedef struct uint128_t {
  uint64_t _a, _b;
} uint128_t;

uint8_t dwcas_compare_exchange_128(
  uint128_t* dest,
  void* old_1,
  uint64_t old_2,
  void* new_1,
  uint64_t new_2
);

#endif /* CONQUER_POINTER_DWCAS_H */

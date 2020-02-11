#ifndef CONQUER_POINTER_DWCAS_H
#define CONQUER_POINTER_DWCAS_H

#include <stdint.h>

struct dwcas_uint128_t {
  uint64_t first, second;
};

uint8_t dwcas_compare_exchange_128(
  struct dwcas_uint128_t* dest,
  struct dwcas_uint128_t old,
  struct dwcas_uint128_t new,
);

#endif /* CONQUER_POINTER_DWCAS_H */

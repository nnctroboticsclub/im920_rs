#pragma once

#include <functional>
#include <cstring>

#include <srobo2/ffi/base.hpp>
#include <srobo2/ffi/im920.hpp>
#include <logger/logger.hpp>

namespace srobo2::com {
class CIM920 {
  srobo2::ffi::CIM920* im920_;

  struct Context {
    std::function<void(uint16_t, uint8_t*, size_t)> cb;
  };

  Context context_;

  static void HandleOnData(const void* ctx, uint16_t from, const uint8_t* data,
                           size_t len) {
    auto context = static_cast<const Context*>(ctx);
    context->cb(from, const_cast<uint8_t*>(data), len);
  }

 public:
  CIM920(srobo2::ffi::CStreamTx* tx, srobo2::ffi::CStreamRx* rx,
         srobo2::ffi::CTime* time) {
    im920_ = srobo2::ffi::__ffi_cim920_new(tx, rx, time);
  }

  uint16_t GetNodeNumber(float duration_secs) {
    return srobo2::ffi::__ffi_cim920_get_node_number(im920_, duration_secs);
  }

  bool SetNodeNumber(uint16_t node_number, float duration_secs) {
    return srobo2::ffi::__ffi_cim920_set_node_number(im920_, node_number,
                                                     duration_secs);
  }

  uint32_t GetGroupNumber(float duration_secs) {
    return srobo2::ffi::__ffi_cim920_get_group_number(im920_, duration_secs);
  }

  uint8_t GetChannel(float duration_secs) {
    return srobo2::ffi::__ffi_cim920_get_channel(im920_, duration_secs);
  }

  bool SetChannel(uint8_t channel, float duration_secs) {
    return srobo2::ffi::__ffi_cim920_set_channel(im920_, channel,
                                                 duration_secs);
  }

  bool EnableWrite(float duration_secs) {
    return srobo2::ffi::__ffi_cim920_enable_write(im920_, duration_secs);
  }

  std::string GetVersion(float duration_secs) {
    auto ptr = srobo2::ffi::__ffi_cim920_get_version(im920_, duration_secs);
    if (ptr == nullptr) {
      return "";
    }

    auto len = std::strlen((const char*)ptr);

    return std::string((const char*)ptr, len);
  }

  void OnData(std::function<void(uint16_t, uint8_t*, size_t)> cb) {
    Context context = {cb};
    context_ = context;

    srobo2::ffi::__ffi_cim920_on_data(im920_, &HandleOnData, &context_);
  }

  bool Send(uint16_t dest, const uint8_t* data, size_t len,
            float duration_secs) {
    return srobo2::ffi::__ffi_cim920_transmit_delegate(im920_, dest, data, len,
                                                       duration_secs);
  };
};
}  // namespace srobo2::com
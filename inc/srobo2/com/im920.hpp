#pragma once
#include <srobo2/ffi/base.hpp>
#include <srobo2/ffi/im920.hpp>

robotics::logger::Logger logger{"connectTs", "connectTs"};

namespace srobo2::com {
class CIM920 {
  srobo2::ffi::CIM920* im920_;

 public:
  CIM920(srobo2::ffi::CStreamTx* tx, srobo2::ffi::CStreamRx* rx,
         srobo2::ffi::CTime* time) {
    im920_ = srobo2::ffi::__ffi_cim920_new(tx, rx, time);
  }

  uint16_t GetNodeNumber(float duration_secs) {
    return srobo2::ffi::__ffi_cim920_get_node_number(im920_, duration_secs);
  }

  std::string GetVersion(float duration_secs) {
    auto ptr = srobo2::ffi::__ffi_cim920_get_version(im920_, duration_secs);
    if (ptr == nullptr) {
      return "";
    }

    auto len = std::strlen((const char*)ptr);

    return std::string((const char*)ptr, len);
  }

  struct Context {
    std::function<void(uint16_t, uint8_t*, size_t)> cb;
  };

  static void HandleOnData(const void* ctx, uint16_t from, const uint8_t* data,
                           size_t len) {
    auto context = static_cast<const Context*>(ctx);
    context->cb(from, const_cast<uint8_t*>(data), len);
  }

  void OnData(std::function<void(uint16_t, uint8_t*, size_t)> cb) {
    Context context = {cb};
    srobo2::ffi::__ffi_cim920_on_data(im920_, &HandleOnData, &context);
  }

  void Send(uint16_t dest, const uint8_t* data, size_t len,
            float duration_secs) {
    srobo2::ffi::__ffi_cim920_transmit_delegate(im920_, dest, data, len,
                                                duration_secs);
  };
};
}  // namespace srobo2::com
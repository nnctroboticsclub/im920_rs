#pragma once

#include <robotics/network/stream.hpp>

#include <srobo2/com/im920.hpp>

namespace srobo2::com {
class IM910_SRobo1 : public robotics::network::Stream<uint8_t, uint16_t> {
  srobo2::com::CIM920 *im920_;

 public:
  IM910_SRobo1(srobo2::com::CIM920 *im920) : im920_(im920) {
    im920->OnData([this](uint16_t from, uint8_t *data, size_t len) {
      this->DispatchOnReceive(from, data, len);
    });
  }

  uint16_t GetNodeNumber() { return im920_->GetNodeNumber(1.0f); }
  uint16_t GetGroupNumber() { return im920_->GetGroupNumber(1.0f); }

  std::string GetVersion() { return im920_->GetVersion(1.0f); }

  void Send(uint16_t dest, uint8_t *data, uint32_t len) override {
    im920_->Send(dest, data, len, 1.0f);
  }
};
}  // namespace srobo2::com
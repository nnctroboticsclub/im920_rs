#pragma once

#include <robotics/network/stream.hpp>

#include <srobo2/com/im920.hpp>

namespace srobo2::com {
class IM920_SRobo1 : public robotics::network::Stream<uint8_t, uint16_t> {
  srobo2::com::CIM920 *im920_;

 public:
  IM920_SRobo1(srobo2::com::CIM920 *im920) : im920_(im920) {
    im920->OnData([this](uint16_t from, uint8_t *data, size_t len) {
      this->DispatchOnReceive(from, data, len);
    });
  }

  uint16_t GetNodeNumber() { return im920_->GetNodeNumber(0.05f); }
  uint32_t GetGroupNumber() { return im920_->GetGroupNumber(0.05f); }

  std::string GetVersion() { return im920_->GetVersion(0.05f); }

  void Send(uint16_t dest, uint8_t *data, uint32_t len) override {
    im920_->Send(dest, data, len, 1.0f);
  }
};
}  // namespace srobo2::com
# keyball-embassy-rp2040

ProMicro RP2040用のRust製Keyballファームウェア(wip)

## 進行状況

- [x] 基本的なキースキャンとUSB HIDへの出力
- [x] PMW3360によるマウス入力とUSB HIDへの出力
- [ ] 分割キーボード間通信(partial)
  - [x] 半二重通信によるバイト列の送受信
  - [x] rkyvによるバイト列のシリアライズ/デシリアライズ
  - [ ] master側でのデータの受信と処理

- [ ] PMW3360のSPI通信の安定化
- [ ] 分割キーボード間通信の安定化

- [ ] 高度なキーマップ機構(レイヤなど)

## CREDITS

これらの先人がいなければここまで作れませんでした。

- もろもろ参考
  - [keyball-rs](https://github.com/hikalium/keyball-rs)
- PMW3360ドライバ関連
  - https://github.com/kndndrj/mouse/tree/8c3cf4707cc392c16c91dc11e53f954f0fd820f1/firmware-rust/mouse-libs/src/pmw3360
- 分割キーボード間半二重通信
  - [QMK Firmware](https://github.com/qmk/qmk_firmware/blob/master/platforms/chibios/drivers/vendor/RP/RP2040/serial_vendor.c)
  - [rusty-dilemma](https://github.com/simmsb/rusty-dilemma/blob/5ffe8f5d2b6b0d534a4309edc737364cd96f44f1/firmware/src/interboard/onewire.rs)
